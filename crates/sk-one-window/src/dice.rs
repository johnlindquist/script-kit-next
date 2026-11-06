//! Deliberate tabletop rolls. A stored row carries both count and sides.
use crate::{Row, quick_tools::NumberDraw};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dice {
    count: u8,
    sides: u8,
}
impl Dice {
    pub fn request(text: &str) -> Option<Self> {
        let q = text.trim().to_ascii_lowercase();
        let (count, sides) = match q.as_str() {
            "roll dice" | "roll the dice" | "dice" => (2, 6),
            "roll a die" | "roll one die" => (1, 6),
            _ => {
                let spec = q.strip_prefix("roll ")?;
                let (a, b) = spec.split_once('d')?;
                (
                    (if a.is_empty() { 1 } else { a.parse().ok()? }),
                    b.parse().ok()?,
                )
            }
        };
        ((1..=10).contains(&count) && (2..=100).contains(&sides)).then_some(Self { count, sides })
    }
    pub fn key(self) -> String {
        format!("quick-dice:{}d{}", self.count, self.sides)
    }
    pub fn row(self) -> Row {
        Row::new(
            self.key(),
            format!("Roll dice · {}d{}", self.count, self.sides),
        )
        .with_sub("Roll once here; type roll 2d8 to choose your dice")
        .with_enter_label("Roll dice")
    }
    pub fn result(self, mut next: impl FnMut() -> u32) -> String {
        let die = NumberDraw::die(self.sides);
        let faces: Vec<i32> = (0..self.count).map(|_| die.draw(&mut next)).collect();
        if faces.len() == 1 {
            return faces[0].to_string();
        }
        let total: i32 = faces.iter().sum();
        format!(
            "{} = {total}",
            faces
                .iter()
                .map(i32::to_string)
                .collect::<Vec<_>>()
                .join(" + ")
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn individual_faces_and_total_are_visible() {
        let mut seq = [0, 5].into_iter();
        assert_eq!(
            Dice::request("roll dice")
                .unwrap()
                .result(|| seq.next().unwrap()),
            "1 + 6 = 7"
        );
        for q in ["roll 0d6", "roll 2d0", "roll 250d6", "roll 2d6 now"] {
            assert!(Dice::request(q).is_none());
        }
    }
}
