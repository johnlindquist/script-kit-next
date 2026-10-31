//! The input destination is a deliberate choice, never a reading of spoken words.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Route {
    #[default]
    Original,
    Kit,
    Markdown,
}
impl Route {
    pub fn toggle_kit(&mut self) {
        *self = if *self == Self::Kit {
            Self::Original
        } else {
            Self::Kit
        };
    }
    pub fn title(self, original: &str) -> String {
        match self {
            Self::Original => format!("Writing in {original}"),
            Self::Kit => "Dictating into Script Kit".into(),
            Self::Markdown => "Review in Markdown".into(),
        }
    }
}
pub fn insert_at(
    text: &str,
    range: std::ops::Range<usize>,
    words: &str,
) -> Option<(String, usize)> {
    if range.start > range.end {
        return None;
    }
    Some((
        format!(
            "{}{}{}",
            text.get(..range.start)?,
            words,
            text.get(range.end..)?
        ),
        range.start + words.len(),
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn route_round_trip_and_unicode_selection() {
        let mut r = Route::default();
        r.toggle_kit();
        assert_eq!(r, Route::Kit);
        r.toggle_kit();
        assert_eq!(r, Route::Original);
        assert_eq!(insert_at("héllo", 1..3, "i"), Some(("hillo".into(), 2)));
        assert!(insert_at("héllo", 2..3, "i").is_none());
    }
}
