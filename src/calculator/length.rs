//! Offline conversions between compatible length units. Only a complete conversion sentence
//! is recognized; unrelated search text and other dimensions keep their ordinary owners.
use super::{format_value, words_for_formatted_number, CalculatorInlineResult};
fn unit(s: &str) -> Option<(f64, &'static str)> {
    Some(match s {
        "mm" | "millimeter" | "millimeters" | "millimetre" | "millimetres" => (0.001, "mm"),
        "cm" | "centimeter" | "centimeters" | "centimetre" | "centimetres" => (0.01, "cm"),
        "m" | "meter" | "meters" | "metre" | "metres" => (1., "m"),
        "km" | "kilometer" | "kilometers" | "kilometre" | "kilometres" => (1000., "km"),
        "mi" | "mile" | "miles" => (1609.344, "mi"),
        "yd" | "yard" | "yards" => (0.9144, "yd"),
        "ft" | "foot" | "feet" => (0.3048, "ft"),
        "in" | "inch" | "inches" => (0.0254, "in"),
        _ => return None,
    })
}
fn number(words: &[&str]) -> Option<f64> {
    const SMALL: [&str; 20] = [
        "zero",
        "one",
        "two",
        "three",
        "four",
        "five",
        "six",
        "seven",
        "eight",
        "nine",
        "ten",
        "eleven",
        "twelve",
        "thirteen",
        "fourteen",
        "fifteen",
        "sixteen",
        "seventeen",
        "eighteen",
        "nineteen",
    ];
    const TENS: [&str; 8] = [
        "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
    ];
    let ten = |s: &str| TENS.iter().position(|&w| w == s).map(|i| (i + 2) * 10);
    match words {
        [s] => s.parse::<f64>().ok().filter(|v| v.is_finite()).or_else(|| {
            SMALL
                .iter()
                .position(|&w| w == *s)
                .or_else(|| ten(s))
                .map(|v| v as f64)
        }),
        [t, s] => Some((ten(t)? + SMALL[1..10].iter().position(|&w| w == *s)? + 1) as f64),
        _ => None,
    }
}
pub(super) fn try_build(input: &str) -> Option<CalculatorInlineResult> {
    if input.len() > 128 || !input.contains(char::is_whitespace) {
        return None;
    }
    let lower = input.trim().to_ascii_lowercase();
    let parts: Vec<_> = lower
        .strip_prefix("convert ")
        .unwrap_or(&lower)
        .split_whitespace()
        .collect();
    let [qty @ .., from, join, to] = parts.as_slice() else {
        return None;
    };
    if !matches!(*join, "to" | "in") {
        return None;
    }
    let (a, _) = unit(from)?;
    let (b, sym) = unit(to)?;
    let n = number(qty)?;
    let value = n * a / b;
    let num = format_value(value)?;
    Some(CalculatorInlineResult {
        raw_input: input.to_string(),
        normalized_expr: format!("{n} * {a} / {b}"),
        operation_name: "Length conversion".into(),
        value,
        formatted: format!("{num} {sym}"),
        words: format!("{} {sym}", words_for_formatted_number(&num)),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spoken_and_numeric_distances_use_exact_international_units() {
        assert_eq!(
            try_build("two miles in kilometers").unwrap().formatted,
            "3.218688 km"
        );
        assert!((try_build("Convert 12 INCHES to feet").unwrap().value - 1.).abs() < 1e-12);
        assert_eq!(try_build("twenty one km to m").unwrap().value, 21000.);
        for s in [
            "notes about two miles in kilometers",
            "2 miles in hours",
            "NaN mi to km",
            "km to m",
        ] {
            assert!(try_build(s).is_none());
        }
    }
}
