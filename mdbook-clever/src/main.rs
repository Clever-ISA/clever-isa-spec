use std::io;

use helpers::StringAppender;
use logos::Logos;
use mdbook::{BookItem, book::Chapter, preprocess::CmdPreprocessor};
use mdbook_fiction_tools::xhtml::{write_rich_node, xml_to_io_error};
use nom::{Finish, combinator};
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use pulldown_cmark_to_cmark::cmark_resume;
use spec_lang::{Error, Spanned, Token, parse::parse_elem};
use xml::{EmitterConfig, EventWriter};

mod helpers;
mod spec_lang;

fn handle_chapter(c: &mut Chapter) -> io::Result<()> {
    let mut content = core::mem::take(&mut c.content);

    let mut parser = Parser::new_ext(&content, Options::all());

    let mut state = None;

    let mut events = Vec::new();

    while let Some(event) = parser.next() {
        match event {
            Event::Start(Tag::CodeBlock(cb)) => match cb {
                CodeBlockKind::Fenced(lang) if lang.trim() == "clever-spec,render" => {
                    state = Some(
                        cmark_resume(events.drain(..), &mut c.content, state.take())
                            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
                    );
                    c.content.push_str("\n\n");
                    let mut body = String::new();

                    while let Some(e) = parser.next() {
                        match e {
                            Event::Text(text) => body.push_str(&text),
                            Event::End(TagEnd::CodeBlock) => break,
                            e => panic!("Got unexpected event {e:?}"),
                        }
                    }

                    let tokens = Token::lexer(&body)
                        .spanned()
                        .map(|(r, s)| r.map(move |t| Spanned { body: t, span: s }))
                        .collect::<Result<Vec<_>, Error>>()
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

                    let elem =
                        combinator::all_consuming(combinator::complete(parse_elem))(&*tokens)
                            .finish()
                            .map(|(_, elem)| elem)
                            .map_err(|e| {
                                io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    format!("Parse Error: {e:?}"),
                                )
                            })?;

                    let rt = elem.body.to_rich_text();

                    let mut writer = StringAppender(&mut c.content);
                    let mut writer = EventWriter::new_with_config(
                        &mut writer,
                        EmitterConfig::new()
                            .write_document_declaration(false)
                            .cdata_to_characters(true),
                    );
                    write_rich_node(&rt, &mut writer).map_err(xml_to_io_error)?;

                    c.content.push_str("\n\n");
                }
                _ => events.push(Event::Start(Tag::CodeBlock(cb))),
            },
            e => events.push(e),
        }
    }

    cmark_resume(events.into_iter(), &mut c.content, state.take())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        .map(|_| ())
}

fn main() -> io::Result<()> {
    let mut args = std::env::args();
    args.next();

    match args.next().as_deref() {
        Some("supports") => return Ok(()),
        Some(s) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unexpected argument {s}"),
            ));
        }
        None => {}
    }

    let (_, mut book) = CmdPreprocessor::parse_input(std::io::stdin())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut err = None;

    book.for_each_mut(|i| match i {
        BookItem::Chapter(c) => err = handle_chapter(c).err().or(err.take()),
        _ => {}
    });

    match err {
        Some(err) => Err(err),
        None => serde_json::to_writer(std::io::stdout(), &book)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e)),
    }
}
