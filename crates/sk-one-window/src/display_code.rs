//! Durable QR data (no network resources or external links) and simple tonal palettes.
use qrcodegen::{QrCode, QrCodeEcc};
use std::fmt::Write;
pub const QUIET: i32 = 4;
pub const MAX_QR_BYTES: usize = 256;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QrGrid {
    pub size: i32,
    cells: Vec<bool>,
}
impl QrGrid {
    pub fn encode(text: &str) -> Option<Self> {
        if text.is_empty() || text.len() > MAX_QR_BYTES {
            return None;
        }
        let qr = QrCode::encode_text(text, QrCodeEcc::Medium).ok()?;
        let size = qr.size();
        let cells = (0..size)
            .flat_map(|y| (0..size).map(move |x| (x, y)))
            .map(|(x, y)| qr.get_module(x, y))
            .collect();
        Some(Self { size, cells })
    }
    pub fn runs(&self, mut visit: impl FnMut(i32, i32, i32)) {
        for y in 0..self.size {
            let mut x = 0;
            while x < self.size {
                if !self.cells[(y * self.size + x) as usize] {
                    x += 1;
                    continue;
                }
                let start = x;
                while x < self.size && self.cells[(y * self.size + x) as usize] {
                    x += 1;
                }
                visit(start + QUIET, y + QUIET, x - start);
            }
        }
    }
    pub fn svg(&self) -> String {
        let side = self.size + 2 * QUIET;
        let mut out = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {side} {side}" shape-rendering="crispEdges"><rect width="{side}" height="{side}" fill="white"/><path fill="black" d=""#
        );
        self.runs(|x, y, n| {
            let _ = write!(out, "M{x} {y}h{n}v1h-{n}z");
        });
        out.push_str("\"/></svg>");
        out
    }
}
pub fn tones(base: u32) -> [u32; 5] {
    let mix = |toward: u32, pct: u32| {
        let c = |shift: u32| {
            let a = (base >> shift) & 255;
            (a * (100 - pct) + toward * pct + 50) / 100
        };
        (c(16) << 16) | (c(8) << 8) | c(0)
    };
    [mix(0, 48), mix(0, 24), base, mix(255, 30), mix(255, 60)]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn palette_preserves_exact_seed_and_qr_bounds() {
        assert_eq!(tones(0x336699)[2], 0x336699);
        assert!(QrGrid::encode("").is_none());
        assert!(QrGrid::encode(&"é".repeat(129)).is_none());
        let grid = QrGrid::encode("Rehearsal 17").unwrap();
        assert!(grid.size >= 21);
        grid.runs(|x, y, n| {
            assert!(x >= 4 && y >= 4 && x + n <= grid.size + 4 && y < grid.size + 4)
        });
        let svg = grid.svg();
        assert!(svg.ends_with("\"/></svg>"));
        assert!(!svg.contains("Rehearsal"));
    }
}
