//! Local launcher readings for the two text sizes Kit already owns. Match a whole request;
//! mentioning a setting in a question, a negation, or text to dictate must not schedule an edit.

pub mod change;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontSurface {
    Editor,
    Terminal,
}

impl FontSurface {
    pub fn key(self) -> &'static str {
        match self {
            Self::Editor => "editorFontSize",
            Self::Terminal => "terminalFontSize",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Editor => "Editor",
            Self::Terminal => "Terminal",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FontPreference {
    pub surface: FontSurface,
    pub pixels: u8,
}

fn surface(words: &[&str]) -> Option<FontSurface> {
    let words = match words {
        ["my" | "the" | "kit" | "kit's", rest @ ..] => rest,
        _ => words,
    };
    match words {
        ["editor"] => Some(FontSurface::Editor),
        ["terminal"] => Some(FontSurface::Terminal),
        _ => None,
    }
}

fn size(words: &[&str]) -> Option<u8> {
    let number = match words {
        [n] => n.strip_suffix("px").unwrap_or(n),
        [n, "px" | "pixels"] => n,
        _ => return None,
    };
    if !number.bytes().all(|c| c.is_ascii_digit()) {
        return None;
    }
    number.parse().ok().filter(|n| (8..=72).contains(n))
}

pub fn read(text: &str) -> Option<FontPreference> {
    if text.len() > 180 {
        return None;
    }
    let text = text
        .trim()
        .trim_end_matches(['.', '!'])
        .to_ascii_lowercase()
        .replace('’', "'");
    let words = text.split_whitespace().collect::<Vec<_>>();
    let w = words.strip_prefix(&["please"]).unwrap_or(&words);
    let (which, pixels) = match w {
        ["set" | "make", rest @ ..] => {
            let at = rest.iter().position(|w| *w == "font" || *w == "text")?;
            let which = surface(&rest[..at])?;
            let tail = &rest[at + 1..];
            let tail = tail.strip_prefix(&["size"]).unwrap_or(tail);
            (which, size(tail.strip_prefix(&["to"]).unwrap_or(tail))?)
        }
        ["use", rest @ ..] => {
            let at = rest.iter().position(|w| *w == "text" || *w == "type")?;
            (
                surface(rest[at + 1..].strip_prefix(&["in"])?)?,
                size(&rest[..at])?,
            )
        }
        _ => return None,
    };
    Some(FontPreference {
        surface: which,
        pixels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_complete_local_request_names_the_surface_and_its_size() {
        for (words, surface, pixels) in [
            ("Set my editor text to 20 pixels", FontSurface::Editor, 20),
            (
                "Please make Kit’s terminal font size 18px!",
                FontSurface::Terminal,
                18,
            ),
            ("Use 22px type in the editor.", FontSurface::Editor, 22),
        ] {
            assert_eq!(read(words), Some(FontPreference { surface, pixels }));
        }
        for words in [
            "Don't set my editor text to 20",
            "Tell Ana: set my editor text to 20",
            "Set my editor font to 18 and turn on the mic",
            "use 20px text in another editor",
            "set the editor font to 200",
            "set editor text to +20",
            "make the font 18",
            "what if I set editor font to 12?",
        ] {
            assert_eq!(read(words), None, "{words}");
        }
    }
}
