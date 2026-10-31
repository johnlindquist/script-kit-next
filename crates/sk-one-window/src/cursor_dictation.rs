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
            Self::Kit => "Use in Kit".into(),
            Self::Markdown => "Review in Markdown".into(),
        }
    }
}
/// A Shift already held at microphone start is not a tap. A Shift used to type
/// another key is not a tap either (the counter observes counts, not characters).
#[derive(Default)]
pub struct ShiftTap {
    previous: Option<bool>,
    down_count: Option<u32>,
}
impl ShiftTap {
    pub fn sample(&mut self, down: bool, keys: u32) -> bool {
        let previous = self.previous.replace(down);
        if previous == Some(false) && down {
            self.down_count = Some(keys);
        }
        previous == Some(true) && !down && self.down_count.take() == Some(keys)
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
    fn shift_only_after_start_and_not_as_part_of_another_chord() {
        let mut s = ShiftTap::default();
        for (d, k, toggles) in [
            (true, 1, false),
            (false, 1, false),
            (true, 1, false),
            (true, 2, false),
            (false, 2, false),
            (true, 2, false),
            (false, 2, true),
            (false, 2, false),
        ] {
            assert_eq!(s.sample(d, k), toggles);
        }
    }
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
