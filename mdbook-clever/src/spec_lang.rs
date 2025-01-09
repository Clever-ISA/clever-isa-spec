use std::{
    borrow::{Borrow, Cow},
    hash::Hash,
    ops::Deref,
};

use logos::{Lexer, Logos, Skip, Span};

#[derive(Clone, Debug)]
pub enum CowArray<'a, T> {
    Borrowed(&'a [T]),
    Owned(Vec<T>),
}

impl<'a, T> Hash for CowArray<'a, T>
where
    T: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}

impl<'a, T, R: AsRef<[T]> + ?Sized> PartialEq<R> for CowArray<'a, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &R) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<'a, T: Eq> Eq for CowArray<'a, T> {}

impl<'a, T> Deref for CowArray<'a, T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(b) => b,
            Self::Owned(v) => v,
        }
    }
}

impl<'a, T> AsRef<[T]> for CowArray<'a, T> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<'a, T> Borrow<[T]> for CowArray<'a, T> {
    fn borrow(&self) -> &[T] {
        self
    }
}

#[derive(Copy, Clone, Default, Debug, Hash, PartialEq, Eq)]
pub struct Extras {
    pub nesting: usize,
}

#[derive(Clone, Default, Debug, Hash, PartialEq, Eq)]
pub struct Spanned<T> {
    pub body: T,
    pub span: Span,
}

#[derive(Clone, Default, Debug, Hash, PartialEq, Eq)]
pub enum Error {
    #[default]
    InvalidToken,
    UnmatchedOpen(Span),
    UnmatchedClose(Span),
    UnterminatedLiteral,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidToken => f.write_str("Unexpceted Input"),
            Self::UnmatchedOpen(c) => f.write_fmt(format_args!(
                "Expected a closing delimiter, got EOF instead at {}:{}",
                c.start, c.end
            )),
            Self::UnmatchedClose(c) => f.write_fmt(format_args!(
                "Got an expected closing delimiter at {}:{}",
                c.start, c.end
            )),
            Self::UnterminatedLiteral => f.write_str("Unexpected EOF before end of literal"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Logos, Clone, Eq, Debug, Hash)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(extras = Extras)]
#[logos(error = Error)]
pub enum Token<'a> {
    #[token("table", priority = 3)]
    KwTable,
    #[token("row", priority = 3)]
    KwRow,
    #[token(":")]
    LabelSep,
    #[token(",")]
    Comma,
    #[token("[", lex_bracket)]
    Bracket(CowArray<'a, Spanned<Token<'a>>>),
    #[token("]", lex_group_end)]
    RightBracket,
    #[token("{", lex_brace)]
    Brace(CowArray<'a, Spanned<Token<'a>>>),
    #[token("}", lex_group_end)]
    RightBrace,
    #[regex("\"([^\"\\\\]|\\[\"'nr\\\\])*+\"", |c| c.slice().strip_prefix("\"").and_then(|c| c.strip_suffix("\"")).ok_or(Error::UnterminatedLiteral))]
    StringLiteral(&'a str),
    #[regex("<!", lex_markdown_literal)]
    MarkdownLiteral(&'a str),
    #[regex("[A-Za-z0-9$_.]+")]
    Identifier(&'a str),
    #[regex("#", lex_comment)]
    Comment,
}

impl<'a, 'b> PartialEq<Token<'a>> for Token<'b> {
    fn eq(&self, other: &Token<'a>) -> bool {
        match (self, other) {
            (Token::KwTable, Token::KwTable) => true,
            (Token::KwRow, Token::KwRow) => true,
            (Token::LabelSep, Token::LabelSep) => true,
            (Token::Comma, Token::Comma) => true,
            (Token::Bracket(c1), Token::Bracket(c2)) => c1 == c2,
            (Token::Brace(c1), Token::Brace(c2)) => c1 == c2,
            (Token::RightBracket, Token::RightBracket) => true,
            (Token::RightBrace, Token::RightBrace) => true,
            (Token::StringLiteral(l1), Token::StringLiteral(l2)) => l1 == l2,
            (Token::MarkdownLiteral(l1), Token::MarkdownLiteral(l2)) => l1 == l2,
            (Token::Identifier(id1), Token::Identifier(id2)) => id1 == id2,
            (Token::Comment, Token::Comment) => true,
            _ => false,
        }
    }
}

impl<'src> Token<'src> {
    pub fn borrowed<'a>(&'a self) -> Token<'a>
    where
        'src: 'a,
    {
        match self {
            Token::KwTable => Token::KwTable,
            Token::KwRow => Token::KwRow,
            Token::LabelSep => Token::LabelSep,
            Token::Comma => Token::Comma,
            Token::Bracket(body) => Token::Bracket(CowArray::Borrowed(body)),
            Token::Brace(body) => Token::Bracket(CowArray::Borrowed(body)),
            Token::RightBracket => Token::RightBracket,
            Token::RightBrace => Token::RightBrace,
            Token::StringLiteral(body) => Token::StringLiteral(body),
            Token::MarkdownLiteral(body) => Token::MarkdownLiteral(body),
            Token::Identifier(id) => Token::Identifier(id),
            Token::Comment => Token::Comment,
        }
    }
}

fn lex_comment<'src>(l: &mut Lexer<'src, Token<'src>>) -> Skip {
    let remaining = l.remainder();

    l.bump(remaining.find('\n').unwrap_or(remaining.len()));

    Skip
}

fn lex_group_end<'src>(l: &mut Lexer<'src, Token<'src>>) -> Result<(), Error> {
    if l.extras.nesting == 0 {
        Err(Error::UnmatchedClose(l.span()))
    } else {
        l.extras.nesting -= 1;
        Ok(())
    }
}

fn lex_brace<'src>(
    l: &mut Lexer<'src, Token<'src>>,
) -> Result<CowArray<'src, Spanned<Token<'src>>>, Error> {
    let nesting = l.extras.nesting;
    l.extras.nesting += 1;
    let mut v = Vec::new();
    let start_span = l.span();
    while let Some(tok) = l.next() {
        let tok = tok?;
        match tok {
            Token::RightBrace => {
                if l.extras.nesting == nesting {
                    return Ok(CowArray::Owned(v));
                } else {
                    return Err(Error::UnmatchedClose(l.span()));
                }
            }
            tok => v.push(Spanned {
                body: tok,
                span: l.span(),
            }),
        }
    }
    Err(Error::UnmatchedOpen(start_span))
}

fn lex_bracket<'src>(
    l: &mut Lexer<'src, Token<'src>>,
) -> Result<CowArray<'src, Spanned<Token<'src>>>, Error> {
    let nesting = l.extras.nesting;
    l.extras.nesting += 1;
    let mut v = Vec::new();
    let start_span = l.span();
    while let Some(tok) = l.next() {
        let tok = tok?;
        match tok {
            Token::RightBracket => {
                if l.extras.nesting == nesting {
                    return Ok(CowArray::Owned(v));
                } else {
                    return Err(Error::UnmatchedClose(l.span()));
                }
            }
            tok => v.push(Spanned {
                body: tok,
                span: l.span(),
            }),
        }
    }
    Err(Error::UnmatchedOpen(start_span))
}

fn lex_markdown_literal<'source>(
    l: &mut Lexer<'source, Token<'source>>,
) -> Result<&'source str, Error> {
    let sl = l.remainder();
    let mut level = 0;

    let mut iter = sl.char_indices();

    while let Some((idx, c)) = iter.next() {
        match c {
            '<' => match iter.next().ok_or(Error::InvalidToken)? {
                (_, '!') => level += 1,
                _ => {}
            },
            '!' => match iter.next().ok_or(Error::InvalidToken)? {
                (_, '>') => {
                    if level == 0 {
                        l.bump(idx + 2);

                        return Ok(&sl[..idx]);
                    } else {
                        level -= 1;
                    }
                }
                _ => {}
            },
            '\\' => {
                iter.next().ok_or(Error::UnterminatedLiteral)?;
            }
            _ => {}
        }
    }

    Err(Error::UnterminatedLiteral)
}

pub mod ast;

pub mod parse;
