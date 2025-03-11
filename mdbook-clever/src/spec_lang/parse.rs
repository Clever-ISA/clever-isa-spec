use core::slice;
use std::borrow::Cow;

use logos::Span;
use mdbook_fiction_tools::bookir;
use parse_error::{ErrorContext, Expectation};

use super::{
    Spanned, Token,
    ast::{Array, Elem, IPath, Id, Table},
};

pub mod parse_error;

pub use parse_error::Error;

use nom::{
    Parser,
    branch::alt,
    combinator::{all_consuming, complete, cut, map_opt, opt},
    error::context,
    multi, sequence,
};

pub type Input<'src, 'a> = &'a [Spanned<Token<'src>>];

pub type IResult<'src, 'a, O> = nom::IResult<Input<'src, 'a>, Spanned<O>, Error<Input<'src, 'a>>>;

pub fn tag<'src, T: PartialEq<Token<'src>> + Into<ErrorContext> + Clone>(
    tag: T,
) -> impl for<'a> FnMut(Input<'src, 'a>) -> IResult<'src, 'a, Token<'a>> {
    move |input| match input {
        [Spanned { body, span }, rest @ ..] if &tag == body => Ok((
            rest,
            Spanned {
                body: body.borrowed(),
                span: span.clone(),
            },
        )),
        span => Err(nom::Err::Error(Error::create(span, tag.clone().into()))),
    }
}

pub fn commit_after<I, O, E, F>(
    mut n: usize,
    mut f: F,
) -> impl for<'a> FnMut(I) -> nom::IResult<I, O, E>
where
    F: nom::Parser<I, O, E>,
{
    move |i| {
        let commit = n == 0;
        n = n - 1;
        match f.parse(i) {
            Ok(v) => Ok(v),
            Err(nom::Err::Error(e)) if commit => Err(nom::Err::Failure(e)),
            Err(e) => Err(e),
        }
    }
}

pub fn parse_id<'src, 'a>(input: Input<'src, 'a>) -> IResult<'src, 'a, Id<'src>> {
    match input {
        [
            Spanned {
                body: Token::Identifier(id),
                span,
            },
            rest @ ..,
        ] => Ok((
            rest,
            Spanned {
                body: Id(id),
                span: span.clone(),
            },
        )),
        rest => Err(nom::Err::Error(Error::create(
            rest,
            Expectation::Identifier,
        ))),
    }
}

pub fn parse_path<'src, 'a>(input: Input<'src, 'a>) -> IResult<'src, 'a, IPath<'src>> {
    multi::separated_list1(tag(Token::LabelSep), commit_after(1, parse_id))(input).map(|(i, v)| {
        let begin_span = &v.first().expect("list must have an element").span;
        let end_span = &v.last().expect("list must have an element").span;

        let span = Span {
            start: begin_span.start,
            end: end_span.end,
        };

        (
            i,
            Spanned {
                body: IPath(v),
                span,
            },
        )
    })
}

enum EscapeErr<'src> {
    Done(&'src str),
    EscapeError(usize),
}

fn escape_markdown<'src>(
    body: &mut Option<String>,
    mut lit: &'src str,
    mut pos: usize,
) -> Result<(&'src str, usize), EscapeErr<'src>> {
    let escape = lit.find('\\').ok_or_else(|| EscapeErr::Done(lit))?;
    pos += escape;
    let (l, r) = lit.split_at(escape);
    let st = body.get_or_insert_with(String::new);
    st.push_str(l);
    let seq;
    (seq, lit) = r.split_at(2);

    match seq {
        "\\!" => st.push('!'),
        "\\n" => st.push('\n'),
        "\\r" => st.push('\r'),
        "\\\\" => st.push('\\'),
        r @ ("\\<" | "\\[" | "\\>" | "\\]" | "\\)" | "\\(" | "\\#") => st.push_str(r),
        n => {
            return Err(EscapeErr::EscapeError(pos));
        }
    }
    Ok((lit, pos + 2))
}

pub fn parse_markdown<'src, 'a>(input: Input<'src, 'a>) -> IResult<'src, 'a, Cow<'src, str>> {
    match input {
        [
            Spanned {
                body: Token::MarkdownLiteral(lit),
                span,
            },
            rest @ ..,
        ] => {
            let span = span.clone();
            let mut lit = *lit;
            let mut owned_body = None;
            let mut pos = span.start;
            if lit.starts_with("!\n") {
                owned_body = Some(String::new());
                let lit = lit[2..].trim_start_matches('\n');
                let prefix_end = lit
                    .find(|c: char| !c.is_whitespace() || c == '\n')
                    .unwrap_or(lit.len());
                let prefix = &lit[..prefix_end];

                for line in lit.lines() {
                    let mut line = line.strip_prefix(prefix).unwrap_or(line.trim_start());
                    let rest = loop {
                        match escape_markdown(&mut owned_body, line, pos) {
                            Ok(r) => (line, pos) = r,
                            Err(EscapeErr::Done(r)) => break r,
                            Err(EscapeErr::EscapeError(pos)) => {
                                return Err(nom::Err::Error(Error::create(
                                    input,
                                    ErrorContext::EscapeError(pos),
                                )));
                            }
                        }
                    };

                    if let Some(owned_body) = owned_body.as_mut() {
                        owned_body.push_str(rest);
                        owned_body.push('\n');
                    }
                }
            } else {
                let rest = loop {
                    match escape_markdown(&mut owned_body, lit, pos) {
                        Ok(r) => (lit, pos) = r,
                        Err(EscapeErr::Done(r)) => break r,
                        Err(EscapeErr::EscapeError(pos)) => {
                            return Err(nom::Err::Error(Error::create(
                                input,
                                ErrorContext::EscapeError(pos),
                            )));
                        }
                    }
                };

                if let Some(owned_body) = owned_body.as_mut() {
                    owned_body.push_str(rest)
                }
            };

            let body = match owned_body {
                Some(owned) => Cow::Owned(owned),
                None => Cow::Borrowed(lit),
            };

            Ok((rest, Spanned { body, span }))
        }
        input => Err(nom::Err::Error(Error::create(
            input,
            Expectation::StringLiteral,
        ))),
    }
}

pub fn parse_str<'src, 'a>(input: Input<'src, 'a>) -> IResult<'src, 'a, Cow<'src, str>> {
    match input {
        [
            Spanned {
                body: Token::StringLiteral(lit),
                span,
            },
            rest @ ..,
        ] => {
            let span = span.clone();
            let mut lit = *lit;
            let mut owned_body = None;
            let mut pos = span.start;

            while let Some(escape) = lit.find('\\') {
                pos += escape;
                let (l, r) = lit.split_at(escape);
                let mut st = owned_body.get_or_insert_with(String::new);
                st.push_str(l);
                let seq;
                (seq, lit) = r.split_at(2);
                pos += 2;

                match seq {
                    "\\\"" => st.push('"'),
                    "\\n" => st.push('\n'),
                    "\\r" => st.push('\r'),
                    "\\\\" => st.push('\\'),
                    "\\'" => st.push('\''),
                    n => {
                        return Err(nom::Err::Failure(Error::create(
                            &input[..1],
                            ErrorContext::EscapeError(pos),
                        )));
                    }
                }
            }

            if let Some(mut owned) = owned_body {
                owned.push_str(lit);
                Ok((
                    rest,
                    Spanned {
                        body: Cow::Owned(owned),
                        span,
                    },
                ))
            } else {
                Ok((
                    rest,
                    Spanned {
                        body: Cow::Borrowed(lit),
                        span,
                    },
                ))
            }
        }
        input => Err(nom::Err::Error(Error::create(
            input,
            Expectation::StringLiteral,
        ))),
    }
}

pub fn map_spanned<'src, P, O, O2, I, E, F>(
    mut parser: P,
    mut map: F,
) -> impl FnMut(I) -> nom::IResult<I, Spanned<O>, E>
where
    P: Parser<I, Spanned<O2>, E>,
    F: FnMut(O2) -> O,
{
    move |input| match parser.parse(input) {
        Ok((rest, Spanned { body, span })) => Ok((
            rest,
            Spanned {
                body: map(body),
                span,
            },
        )),
        Err(e) => Err(e),
    }
}

pub fn bracket<'src, P: 'src, O>(
    body_parser: P,
) -> impl for<'a> FnMut(Input<'src, 'a>) -> IResult<'src, 'a, Vec<Spanned<O>>> + 'src
where
    P: for<'a> Parser<Input<'src, 'a>, Spanned<O>, Error<Input<'src, 'a>>> + Clone,
{
    move |input| match input {
        [
            Spanned {
                body: Token::Bracket(body),
                span,
            },
            rest @ ..,
        ] => cut(all_consuming(complete(multi::separated_list0(
            tag(Token::Comma),
            body_parser.clone(),
        ))))(body)
        .map(|(_, val)| {
            (
                rest,
                Spanned {
                    body: val,
                    span: span.clone(),
                },
            )
        })
        .map_err(|e| {
            e.map(|e| e.with_context_and_input(input, ErrorContext::Context("In brace group")))
        }),
        i => Err(nom::Err::Error(Error::create(i, Expectation::Group))),
    }
}

pub fn brace<'src, P: 'src, O>(
    body_parser: P,
) -> impl for<'a> FnMut(Input<'src, 'a>) -> IResult<'src, 'a, Vec<Spanned<O>>> + 'src
where
    P: for<'a> Parser<Input<'src, 'a>, Spanned<O>, Error<Input<'src, 'a>>> + Clone,
{
    move |input| match input {
        [
            Spanned {
                body: Token::Brace(body),
                span,
            },
            rest @ ..,
        ] => cut(all_consuming(complete(multi::separated_list0(
            tag(Token::Comma),
            body_parser.clone(),
        ))))(body)
        .map(|(_, val)| {
            (
                rest,
                Spanned {
                    body: val,
                    span: span.clone(),
                },
            )
        })
        .map_err(|e| {
            e.map(|e| e.with_context_and_input(input, ErrorContext::Context("In brace group")))
        }),
        i => Err(nom::Err::Error(Error::create(i, Expectation::Group))),
    }
}

pub fn parse_table<'src, 'a>(input: Input<'src, 'a>) -> IResult<'src, 'a, Table<'src>> {
    let (
        rest,
        (
            a,
            label,
            head,
            Spanned {
                body: rows,
                span: end_span,
            },
        ),
    ) = sequence::tuple((
        tag(Token::KwTable),
        cut(parse_path),
        opt(map_spanned(bracket(parse_elem), Array)),
        cut(brace(parse_row)),
    ))(input)?;

    let a: Spanned<Token> = a;

    let span = Span {
        start: a.span.start,
        end: end_span.end,
    };

    Ok((
        rest,
        Spanned {
            body: Table {
                label,
                heading: head,
                rows,
            },
            span,
        },
    ))
}

pub fn parse_row<'src, 'a>(input: Input<'src, 'a>) -> IResult<'src, 'a, Array<'src>> {
    let (rest, (t, body)) =
        sequence::tuple((tag(Token::KwRow), map_spanned(bracket(parse_elem), Array)))(input)
            .map_err(|e| e.map(|e| e.with_context(ErrorContext::Context("Parsing row here"))))?;
    let a: Spanned<Token> = t;

    let span = Span {
        start: a.span.start,
        end: body.span.end,
    };

    Ok((rest, body))
}

pub fn parse_elem<'src, 'a>(input: Input<'src, 'a>) -> IResult<'src, 'a, Elem<'src>> {
    alt((
        map_spanned(parse_table, Elem::Table),
        map_spanned(parse_markdown, Elem::MarkdownLiteral),
        map_spanned(parse_str, Elem::StringLiteral),
    ))(input)
}
