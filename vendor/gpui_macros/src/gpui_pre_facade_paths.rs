// Modified for gpui-pre (snapshot of zed@bcf6582): this module is added by the script.
// Injected by gpui-kit's script/bump-gpui.ts. Keep fixes in that script.
use proc_macro::{Group, Ident, Literal, Punct, Spacing, TokenStream, TokenTree};
use proc_macro_crate::{crate_name, FoundCrate};

enum FacadePath {
    Itself,
    Name(String),
}

pub(crate) fn rewrite(stream: TokenStream) -> TokenStream {
    let facade = match crate_name("gpui-kit") {
        Ok(FoundCrate::Name(name)) => Some(FacadePath::Name(name)),
        Ok(FoundCrate::Itself) => Some(FacadePath::Itself),
        Err(_) => None,
    };
    rewrite_stream(stream, facade.as_ref())
}

fn rewrite_stream(stream: TokenStream, facade: Option<&FacadePath>) -> TokenStream {
    let tokens: Vec<_> = stream.into_iter().collect();
    let mut output = TokenStream::new();
    for (index, token) in tokens.iter().enumerate() {
        match token {
            TokenTree::Group(group) => {
                let mut rewritten = Group::new(group.delimiter(), rewrite_stream(group.stream(), facade));
                rewritten.set_span(group.span());
                output.extend([TokenTree::Group(rewritten)]);
            }
            TokenTree::Ident(ident)
                if is_path_head(&tokens, index)
                    && (ident.to_string() == "gpui" || ident.to_string() == "gpui_platform") =>
            {
                append_path_head(&mut output, ident, facade);
            }
            TokenTree::Literal(literal) => {
                output.extend([TokenTree::Literal(rewrite_literal(literal, facade))]);
            }
            token => output.extend([token.clone()]),
        }
    }
    output
}

fn is_path_head(tokens: &[TokenTree], index: usize) -> bool {
    matches!(tokens.get(index + 1), Some(TokenTree::Punct(first)) if first.as_char() == ':')
        && matches!(tokens.get(index + 2), Some(TokenTree::Punct(second)) if second.as_char() == ':')
}

fn append_path_head(output: &mut TokenStream, original: &Ident, facade: Option<&FacadePath>) {
    let name = match facade {
        Some(FacadePath::Itself) => "crate",
        Some(FacadePath::Name(name)) => name,
        None => {
            output.extend([TokenTree::Ident(original.clone())]);
            return;
        }
    };
    output.extend([TokenTree::Ident(Ident::new(name, original.span()))]);
    if original.to_string() == "gpui_platform" {
        output.extend([
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("platform", original.span())),
        ]);
    }
}

fn rewrite_literal(literal: &Literal, facade: Option<&FacadePath>) -> Literal {
    let (gpui, platform) = match facade {
        Some(FacadePath::Itself) => ("crate::".into(), "crate::platform::".into()),
        Some(FacadePath::Name(name)) =>
            (format!("::{name}::"), format!("::{name}::platform::")),
        None => return literal.clone(),
    };
    let text = literal.to_string();
    let rewritten = text
        .replace("::gpui_platform::", &platform)
        .replace("::gpui::", &gpui);
    if rewritten == text { return literal.clone() }
    let Ok(parsed) = rewritten.parse::<TokenStream>() else { return literal.clone() };
    let mut tokens = parsed.into_iter();
    match (tokens.next(), tokens.next()) {
        (Some(TokenTree::Literal(literal)), None) => literal,
        _ => literal.clone(),
    }
}
