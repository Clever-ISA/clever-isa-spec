use std::{fmt::Write, io};

use pulldown_cmark::{BrokenLinkCallback, CowStr};

pub struct StringAppender<'a>(pub &'a mut String);

impl<'a> io::Write for StringAppender<'a> {
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match core::str::from_utf8(buf) {
            Ok(st) => {
                self.0.push_str(st);
                Ok(st.len())
            }
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        match core::str::from_utf8(buf) {
            Ok(st) => Ok(self.0.push_str(st)),
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> io::Result<()> {
        self.0
            .write_fmt(fmt)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

#[derive(Copy, Clone)]
pub struct TagExpander<'a> {
    base: &'a str,
}

impl<'a> TagExpander<'a> {
    pub const fn new(base_url: &'a str) -> Self {
        Self { base: base_url }
    }

    pub fn resolve_link<'b>(&self, link: CowStr<'b>) -> Option<(CowStr<'b>, CowStr<'b>)> {
        let tag = link.strip_prefix("`")?.strip_suffix("`")?;

        let (group, tail) = tag.split_once("-")?;

        let intermediate = match group {
            "D" => "documents",
            "X" => "extensions",
            "R" => "reports",
            "V" => {
                // TODO;
                return None;
            }
            _ => return None,
        };
        let base = self.base;

        let st = format!("{base}/{intermediate}/{tail}.md");

        Some((CowStr::from(st), CowStr::from(tag).into_static()))
    }
}

impl<'input, 'a> BrokenLinkCallback<'input> for TagExpander<'a> {
    fn handle_broken_link(
        &mut self,
        link: pulldown_cmark::BrokenLink<'input>,
    ) -> Option<(
        pulldown_cmark::CowStr<'input>,
        pulldown_cmark::CowStr<'input>,
    )> {
        eprintln!("handle_broken_link({link:?})");
        self.resolve_link(link.reference)
    }
}
