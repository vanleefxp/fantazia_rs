//! Extraction of the literal text from a macro invocation.
//!
//! Every macro accepts either a single string literal or a sequence of tokens that is rendered
//! back into the textual form understood by the `fantazia_edo12` parsers. This is what makes
//! `opitch!(F+)` and `opitch!("F+")` equivalent: both produce the source `"F+"`.

use proc_macro2::{Delimiter, Span, TokenStream, TokenTree};

/// The literal text of a macro invocation together with the span it came from.
pub(crate) struct Source {
    pub(crate) text: String,
    pub(crate) span: Span,
}

impl Source {
    /// Parses the literal text as an `i8` when it consists of a number only.
    ///
    /// Both plain integers (`-3`) and integer literals with a suffix (`3i8`) are accepted, so the
    /// macros keep working when a numeric argument is written with an explicit type.
    pub(crate) fn as_i8(&self) -> Option<i8> {
        self.text.parse().ok().or_else(|| {
            syn::parse_str::<syn::LitInt>(&self.text)
                .ok()
                .and_then(|literal| literal.base10_parse().ok())
        })
    }
}

/// Reads the literal of a macro invocation, supporting both the bare and the string form.
pub(crate) fn parse(input: TokenStream) -> syn::Result<Source> {
    if let Ok(lit) = syn::parse2::<syn::LitStr>(input.clone()) {
        return Ok(Source {
            text: lit.value(),
            span: lit.span(),
        });
    }

    let span = input
        .clone()
        .into_iter()
        .next()
        .map(|token| token.span())
        .unwrap_or_else(Span::call_site);

    if input.is_empty() {
        return Err(syn::Error::new(
            span,
            "expected a literal, e.g. `C_1` or `\"C_1\"`",
        ));
    }

    let mut text = String::new();
    render(input, &mut text);
    Ok(Source { text, span })
}

/// Renders a token stream back into the notation accepted by the `fantazia_edo12` parsers.
///
/// Whitespace between tokens is insignificant for those parsers, so identifiers, punctuation and
/// literals are concatenated directly. Groups keep their original delimiters so that bracketed
/// accidentals such as `[+3]` survive the round trip.
fn render(stream: TokenStream, out: &mut String) {
    for token in stream {
        match token {
            TokenTree::Group(group) => {
                let (open, close) = match group.delimiter() {
                    Delimiter::Parenthesis => ("(", ")"),
                    Delimiter::Bracket => ("[", "]"),
                    Delimiter::Brace => ("{", "}"),
                    Delimiter::None => ("", ""),
                };
                out.push_str(open);
                render(group.stream(), out);
                out.push_str(close);
            }
            TokenTree::Ident(ident) => out.push_str(&ident.to_string()),
            TokenTree::Punct(punct) => out.push(punct.as_char()),
            TokenTree::Literal(literal) => out.push_str(&literal.to_string()),
        }
    }
}
