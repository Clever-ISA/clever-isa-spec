use std::borrow::Cow;

use mdbook_fiction_tools::{
    bookir::{self, Alignment, RichText, RichTextOptions, RichTextParser, TableRow},
    xhtml::write_rich_node,
};
use xml::{EventWriter, writer::XmlEvent};

use super::Spanned;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Elem<'src> {
    Table(Table<'src>),
    MarkdownLiteral(Cow<'src, str>),
    StringLiteral(Cow<'src, str>),
}

impl<'src> Elem<'src> {
    pub fn to_rich_text<'a>(&'a self) -> RichText<'a> {
        match self {
            Elem::Table(table) => {
                let mut head = table
                    .heading
                    .as_ref()
                    .map(|v| {
                        v.body
                            .0
                            .iter()
                            .map(|e| e.body.to_rich_text())
                            .collect::<Vec<_>>()
                    })
                    .map(|body| TableRow { elems: body });

                let mut rows = table
                    .rows
                    .iter()
                    .map(|v| {
                        v.body
                            .0
                            .iter()
                            .map(|e| e.body.to_rich_text())
                            .collect::<Vec<_>>()
                    })
                    .map(|body| TableRow { elems: body })
                    .collect::<Vec<_>>();

                let len = head
                    .as_ref()
                    .or_else(|| rows.first())
                    .map(|v| v.elems.len())
                    .unwrap_or(0);

                let align = vec![Alignment::None; len]; // Handle custom alignments later if needed

                let table = bookir::Table {
                    align,
                    head,
                    body: rows,
                };

                RichText::Table(table)
            }
            Elem::MarkdownLiteral(st) => RichText::Paragraph(
                RichTextParser::new(st, RichTextOptions {
                    math: false,
                    ..Default::default()
                })
                .collect(),
            ),
            Elem::StringLiteral(st) => RichText::RawText(st.into()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Table<'src> {
    pub label: Spanned<IPath<'src>>,
    pub heading: Option<Spanned<Array<'src>>>,
    pub rows: Vec<Spanned<Array<'src>>>,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Id<'src>(pub &'src str);

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct IPath<'src>(pub Vec<Spanned<Id<'src>>>);

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Array<'src>(pub Vec<Spanned<Elem<'src>>>);
