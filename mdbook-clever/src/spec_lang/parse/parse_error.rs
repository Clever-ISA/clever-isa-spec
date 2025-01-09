use logos::Span;
use nom::error::{ContextError, ParseError};

use crate::spec_lang::Token;

use super::Input;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Expectation {
    Identifier,
    Group,
    StringLiteral,
    MarkdownLiteral,
    LitToken(Token<'static>),
}

impl core::fmt::Display for Expectation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for Expectation {}

impl<'src> PartialEq<Token<'src>> for Expectation {
    fn eq(&self, other: &Token<'src>) -> bool {
        match (self, other) {
            (Expectation::Identifier, Token::Identifier(_)) => true,
            (Expectation::Group, Token::Bracket(_)) => true,
            (Expectation::StringLiteral, Token::StringLiteral(_)) => true,
            (Expectation::MarkdownLiteral, Token::MarkdownLiteral(_)) => true,
            (Expectation::LitToken(tok), tok2) => tok.borrowed() == tok2.borrowed(),
            _ => false,
        }
    }
}

impl<'src> PartialEq<Expectation> for Token<'src> {
    fn eq(&self, other: &Expectation) -> bool {
        other == self
    }
}

impl<'src, R: ?Sized> PartialEq<R> for Token<'src>
where
    R: AsRef<[Expectation]>,
{
    fn eq(&self, other: &R) -> bool {
        other.as_ref().iter().any(|v| v == self)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ErrorContext {
    Expectation(Vec<Expectation>),
    Context(&'static str),
    EscapeError(usize),
    Nom(nom::error::ErrorKind),
    Span(Span),
}

impl From<Expectation> for ErrorContext {
    fn from(value: Expectation) -> Self {
        ErrorContext::Expectation(vec![value])
    }
}

impl<V: Into<Vec<Expectation>>> From<V> for ErrorContext {
    fn from(value: V) -> Self {
        ErrorContext::Expectation(value.into())
    }
}

impl From<Token<'static>> for ErrorContext {
    fn from(value: Token<'static>) -> Self {
        ErrorContext::Expectation(vec![Expectation::LitToken(value)])
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Error<I> {
    pub primary: (I, ErrorContext),
    pub context: Vec<(Option<I>, ErrorContext)>,
}

impl<I> Error<I> {
    pub fn create<C: Into<ErrorContext>>(input: I, ctx: C) -> Self {
        Self {
            primary: (input, ctx.into()),
            context: Vec::new(),
        }
    }

    pub fn push_context(&mut self, ctx: ErrorContext) {
        self.context.push((None, ctx))
    }

    pub fn push_context_with_input(&mut self, input: I, ctx: ErrorContext) {
        self.context.push((Some(input), ctx));
    }

    pub fn with_context(mut self, ctx: ErrorContext) -> Self {
        self.push_context(ctx);
        self
    }

    pub fn with_context_and_input(mut self, input: I, ctx: ErrorContext) -> Self {
        self.push_context_with_input(input, ctx);
        self
    }

    pub fn contexts(self) -> impl IntoIterator<Item = (Option<I>, ErrorContext)> {
        core::iter::once(self.primary).map(|(a, b)| (Some(a), b))
    }
}

impl<I> Extend<(Option<I>, ErrorContext)> for Error<I> {
    fn extend<T: IntoIterator<Item = (Option<I>, ErrorContext)>>(&mut self, iter: T) {
        self.context.extend(iter)
    }
}

impl<I> Extend<(I, ErrorContext)> for Error<I> {
    fn extend<T: IntoIterator<Item = (I, ErrorContext)>>(&mut self, iter: T) {
        self.context
            .extend(iter.into_iter().map(|(a, b)| (Some(a), b)))
    }
}

impl<I> ParseError<I> for Error<I> {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        Self::create(input, ErrorContext::Nom(kind))
    }

    fn append(input: I, kind: nom::error::ErrorKind, other: Self) -> Self {
        other.with_context_and_input(input, ErrorContext::Nom(kind))
    }
}

impl<I> ContextError<I> for Error<I> {
    fn add_context(input: I, ctx: &'static str, mut other: Self) -> Self {
        other
            .context
            .push((Some(input), ErrorContext::Context(ctx)));
        other
    }
}
