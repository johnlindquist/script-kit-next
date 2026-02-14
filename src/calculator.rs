use regex::Regex;
use std::sync::OnceLock;
use tracing::debug;

const FORMAT_EPSILON: f64 = 1e-12;
const MAX_DECIMAL_WORD_DIGITS: usize = 12;
const SCI_NOTATION_HIGH: f64 = 1e15;
const SCI_NOTATION_LOW: f64 = 1e-9;

#[derive(Clone, Debug, PartialEq)]
pub struct CalculatorInlineResult {
    pub raw_input: String,
    pub normalized_expr: String,
    pub operation_name: String,
    pub value: f64,
    pub formatted: String,
    pub words: String,
}

pub fn try_build(input: &str) -> Option<CalculatorInlineResult> {
    if !looks_like_math(input) {
        debug!("inline calculator rejected input during looks_like_math");
        return None;
    }

    let normalized_expr = normalize_expression(input);
    let value = match meval::eval_str(&normalized_expr) {
        Ok(value) if value.is_finite() => value,
        Ok(value) => {
            debug!(value, "inline calculator rejected non-finite result");
            return None;
        }
        Err(error) => {
            debug!(%error, expression = %normalized_expr, "inline calculator evaluation failed");
            return None;
        }
    };

    let formatted = format_value(value)?;
    let operation_name = infer_operation_name(&normalized_expr);
    let words = words_for_formatted_number(&formatted);

    debug!(
        input = input,
        normalized_expr = %normalized_expr,
        operation_name = %operation_name,
        value,
        formatted = %formatted,
        "inline calculator produced result"
    );

    Some(CalculatorInlineResult {
        raw_input: input.to_string(),
        normalized_expr,
        operation_name,
        value,
        formatted,
        words,
    })
}

fn looks_like_math(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return false;
    }

    if trimmed.contains('"')
        || trimmed.contains('\'')
        || trimmed.contains(':')
        || trimmed.contains('\\')
    {
        debug!(
            input = trimmed,
            "inline calculator rejected disallowed punctuation"
        );
        return false;
    }

    if trimmed.chars().any(|ch| !is_allowed_math_char(ch)) {
        debug!(
            input = trimmed,
            "inline calculator rejected disallowed character"
        );
        return false;
    }

    let lowered = trimmed.to_lowercase();
    let alpha_tokens: Vec<&str> = alpha_token_re()
        .find_iter(&lowered)
        .map(|token| token.as_str())
        .collect();

    for token in &alpha_tokens {
        if !is_allowed_alpha_token(token) {
            debug!(
                token = token,
                "inline calculator rejected unknown alpha token"
            );
            return false;
        }
    }

    let has_digit = lowered.chars().any(|ch| ch.is_ascii_digit());
    let has_pi_or_e_constant = trimmed.contains('π')
        || alpha_tokens
            .iter()
            .any(|token| matches!(*token, "pi" | "e"));
    if !(has_digit || has_pi_or_e_constant) {
        return false;
    }

    let has_operator = trimmed
        .chars()
        .any(|ch| matches!(ch, '+' | '-' | '*' | '/' | '^' | '%' | '×' | '÷' | '−'));
    let has_function = alpha_tokens
        .iter()
        .any(|token| is_supported_function_name(token));
    if !(has_operator || has_function) {
        return false;
    }

    true
}

fn normalize_expression(input: &str) -> String {
    let mut normalized_unicode = String::with_capacity(input.len());
    for ch in input.trim().chars() {
        match ch {
            '×' => normalized_unicode.push('*'),
            '÷' => normalized_unicode.push('/'),
            '−' => normalized_unicode.push('-'),
            'π' => normalized_unicode.push_str("pi"),
            _ => normalized_unicode.push(ch),
        }
    }

    percent_literal_re()
        .replace_all(&normalized_unicode, "($num/100)")
        .to_string()
}

fn infer_operation_name(normalized_expression: &str) -> String {
    let lower = normalized_expression.to_lowercase();

    if contains_call(&lower, "asin") {
        return String::from("Arc Sine");
    }
    if contains_call(&lower, "acos") {
        return String::from("Arc Cosine");
    }
    if contains_call(&lower, "atan") {
        return String::from("Arc Tangent");
    }
    if contains_call(&lower, "sin") {
        return String::from("Sine");
    }
    if contains_call(&lower, "cos") {
        return String::from("Cosine");
    }
    if contains_call(&lower, "tan") {
        return String::from("Tangent");
    }
    if contains_call(&lower, "sqrt") {
        return String::from("Square Root");
    }
    if contains_call(&lower, "abs") {
        return String::from("Absolute Value");
    }
    if contains_call(&lower, "ln") {
        return String::from("Natural Log");
    }
    if contains_call(&lower, "log10") || contains_call(&lower, "log") {
        return String::from("Logarithm");
    }
    if contains_call(&lower, "exp") {
        return String::from("Exponential");
    }
    if contains_call(&lower, "floor") {
        return String::from("Floor");
    }
    if contains_call(&lower, "ceil") {
        return String::from("Ceiling");
    }
    if contains_call(&lower, "round") {
        return String::from("Round");
    }
    if contains_call(&lower, "min") {
        return String::from("Minimum");
    }
    if contains_call(&lower, "max") {
        return String::from("Maximum");
    }

    if lower.contains('+') {
        return String::from("Add");
    }
    if lower.contains('-') {
        return String::from("Subtract");
    }
    if lower.contains('*') {
        return String::from("Multiply");
    }
    if lower.contains('/') {
        return String::from("Divide");
    }
    if lower.contains('^') {
        return String::from("Power");
    }

    String::from("Calculate")
}

fn format_value(value: f64) -> Option<String> {
    if !value.is_finite() {
        return None;
    }

    let normalized = if value.abs() < FORMAT_EPSILON {
        0.0
    } else {
        value
    };
    let abs = normalized.abs();

    if abs > 0.0 && !(SCI_NOTATION_LOW..SCI_NOTATION_HIGH).contains(&abs) {
        return format_scientific(normalized);
    }

    if (normalized.round() - normalized).abs() < FORMAT_EPSILON {
        return Some(format!("{:.0}", normalized.round()));
    }

    let mut fixed = format!("{:.12}", normalized);
    while fixed.ends_with('0') {
        fixed.pop();
    }
    if fixed.ends_with('.') {
        fixed.pop();
    }
    Some(fixed)
}

fn words_for_formatted_number(formatted: &str) -> String {
    let trimmed = formatted.trim();
    if trimmed.is_empty() {
        return String::from("Zero");
    }

    if let Some(rest) = trimmed.strip_prefix('-') {
        let positive_words = words_for_formatted_number(rest);
        return format!("Negative {}", positive_words);
    }

    if let Some((mantissa, exponent)) = split_scientific_notation(trimmed) {
        let mantissa_words = words_for_formatted_number(mantissa);
        let exponent_words = words_for_signed_integer(exponent);
        return format!("{} Times Ten To The {}", mantissa_words, exponent_words);
    }

    let mut parts = trimmed.splitn(2, '.');
    let integer_part = parts.next().unwrap_or("0");
    let mut words = words_for_integer_part(integer_part);

    if let Some(decimal_part) = parts.next() {
        let decimal_digits: String = decimal_part
            .chars()
            .filter(|ch| ch.is_ascii_digit())
            .take(MAX_DECIMAL_WORD_DIGITS)
            .collect();

        if !decimal_digits.is_empty() {
            let decimal_words = decimal_digits
                .chars()
                .map(digit_word)
                .collect::<Vec<_>>()
                .join(" ");
            words.push_str(" Point ");
            words.push_str(&decimal_words);
        }
    }

    words
}

fn is_allowed_math_char(ch: char) -> bool {
    ch.is_ascii_digit()
        || ch.is_ascii_alphabetic()
        || ch.is_ascii_whitespace()
        || matches!(
            ch,
            '.' | ',' | '(' | ')' | '+' | '-' | '*' | '/' | '^' | '%' | '×' | '÷' | '−' | 'π'
        )
}

fn is_allowed_alpha_token(token: &str) -> bool {
    matches!(
        token,
        "pi" | "e"
            | "sin"
            | "cos"
            | "tan"
            | "asin"
            | "acos"
            | "atan"
            | "sqrt"
            | "abs"
            | "ln"
            | "log"
            | "log10"
            | "exp"
            | "floor"
            | "ceil"
            | "round"
            | "min"
            | "max"
    )
}

fn is_supported_function_name(token: &str) -> bool {
    matches!(
        token,
        "sin"
            | "cos"
            | "tan"
            | "asin"
            | "acos"
            | "atan"
            | "sqrt"
            | "abs"
            | "ln"
            | "log"
            | "log10"
            | "exp"
            | "floor"
            | "ceil"
            | "round"
            | "min"
            | "max"
    )
}

fn contains_call(expression: &str, function_name: &str) -> bool {
    let mut search_from = 0;
    while let Some(relative_index) = expression[search_from..].find(function_name) {
        let start = search_from + relative_index;
        let end = start + function_name.len();

        let before_valid = if start == 0 {
            true
        } else {
            !expression[..start]
                .chars()
                .next_back()
                .is_some_and(|ch| ch.is_ascii_alphabetic())
        };
        let after_valid = if end >= expression.len() {
            true
        } else {
            !expression[end..]
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_alphabetic())
        };

        if before_valid && after_valid {
            return true;
        }

        search_from = end;
    }

    false
}

fn format_scientific(value: f64) -> Option<String> {
    let scientific = format!("{:.12e}", value);
    let (mantissa, exponent) = scientific.split_once('e')?;
    let mut cleaned_mantissa = mantissa.to_string();
    while cleaned_mantissa.ends_with('0') {
        cleaned_mantissa.pop();
    }
    if cleaned_mantissa.ends_with('.') {
        cleaned_mantissa.pop();
    }

    let exponent_value = exponent.parse::<i32>().ok()?;
    Some(format!("{}e{}", cleaned_mantissa, exponent_value))
}

fn split_scientific_notation(value: &str) -> Option<(&str, i32)> {
    let e_index = value.find(['e', 'E'])?;
    let (mantissa, exponent_with_e) = value.split_at(e_index);
    let exponent = exponent_with_e.get(1..)?.parse::<i32>().ok()?;
    Some((mantissa, exponent))
}

fn words_for_integer_part(integer_text: &str) -> String {
    let normalized = integer_text.trim_start_matches('+');
    if normalized.is_empty() {
        return String::from("Zero");
    }

    let normalized = normalized.trim_start_matches('0');
    if normalized.is_empty() {
        return String::from("Zero");
    }

    match normalized.parse::<u64>() {
        Ok(value) => number_to_words(value),
        Err(_) => normalized
            .chars()
            .filter(|ch| ch.is_ascii_digit())
            .map(digit_word)
            .collect::<Vec<_>>()
            .join(" "),
    }
}

fn words_for_signed_integer(value: i32) -> String {
    if value < 0 {
        return format!("Negative {}", number_to_words(value.unsigned_abs() as u64));
    }
    number_to_words(value as u64)
}

fn number_to_words(value: u64) -> String {
    if value == 0 {
        return String::from("Zero");
    }

    const SCALES: [&str; 5] = ["", "Thousand", "Million", "Billion", "Trillion"];

    let mut remaining = value;
    let mut scale_index = 0usize;
    let mut chunks: Vec<String> = Vec::new();

    while remaining > 0 {
        let chunk = (remaining % 1000) as u16;
        if chunk > 0 {
            if scale_index >= SCALES.len() {
                return value
                    .to_string()
                    .chars()
                    .map(digit_word)
                    .collect::<Vec<_>>()
                    .join(" ");
            }

            let mut chunk_words = number_below_1000_to_words(chunk);
            if !SCALES[scale_index].is_empty() {
                chunk_words.push(' ');
                chunk_words.push_str(SCALES[scale_index]);
            }
            chunks.push(chunk_words);
        }
        remaining /= 1000;
        scale_index += 1;
    }

    chunks.reverse();
    chunks.join(" ")
}

fn number_below_1000_to_words(value: u16) -> String {
    const ONES: [&str; 10] = [
        "Zero", "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine",
    ];
    const TEENS: [&str; 10] = [
        "Ten",
        "Eleven",
        "Twelve",
        "Thirteen",
        "Fourteen",
        "Fifteen",
        "Sixteen",
        "Seventeen",
        "Eighteen",
        "Nineteen",
    ];
    const TENS: [&str; 10] = [
        "", "", "Twenty", "Thirty", "Forty", "Fifty", "Sixty", "Seventy", "Eighty", "Ninety",
    ];

    let hundreds = value / 100;
    let remainder = value % 100;
    let mut parts: Vec<String> = Vec::new();

    if hundreds > 0 {
        parts.push(format!("{} Hundred", ONES[usize::from(hundreds)]));
    }

    if remainder >= 20 {
        let tens = remainder / 10;
        let ones = remainder % 10;
        if ones == 0 {
            parts.push(TENS[usize::from(tens)].to_string());
        } else {
            parts.push(format!(
                "{} {}",
                TENS[usize::from(tens)],
                ONES[usize::from(ones)]
            ));
        }
    } else if remainder >= 10 {
        parts.push(TEENS[usize::from(remainder - 10)].to_string());
    } else if remainder > 0 {
        parts.push(ONES[usize::from(remainder)].to_string());
    }

    parts.join(" ")
}

fn digit_word(ch: char) -> &'static str {
    match ch {
        '0' => "Zero",
        '1' => "One",
        '2' => "Two",
        '3' => "Three",
        '4' => "Four",
        '5' => "Five",
        '6' => "Six",
        '7' => "Seven",
        '8' => "Eight",
        '9' => "Nine",
        _ => "",
    }
}

fn percent_literal_re() -> &'static Regex {
    static PERCENT_LITERAL_RE: OnceLock<Regex> = OnceLock::new();
    PERCENT_LITERAL_RE.get_or_init(
        || match Regex::new(r"(?P<num>(?:\d+\.\d+|\d+|\.\d+))\s*%") {
            Ok(regex) => regex,
            Err(error) => panic!("SK_CALC_PERCENT_REGEX_INVALID: {}", error),
        },
    )
}

fn alpha_token_re() -> &'static Regex {
    static ALPHA_TOKEN_RE: OnceLock<Regex> = OnceLock::new();
    ALPHA_TOKEN_RE.get_or_init(|| match Regex::new(r"[A-Za-z]+") {
        Ok(regex) => regex,
        Err(error) => panic!("SK_CALC_ALPHA_REGEX_INVALID: {}", error),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_build_returns_none_when_input_is_not_math_like() {
        assert_eq!(None, try_build("hello world"));
    }

    #[test]
    fn test_try_build_returns_none_when_input_contains_disallowed_tokens() {
        assert_eq!(None, try_build("foo(2) + 1"));
    }

    #[test]
    fn test_try_build_evaluates_addition_when_expression_is_valid() {
        let result = try_build("2 + 2").expect("calculator result should be produced");
        assert_eq!("Add", result.operation_name);
        assert_eq!("4", result.formatted);
        assert_eq!("Four", result.words);
        assert!((result.value - 4.0).abs() < 1e-12);
    }

    #[test]
    fn test_try_build_normalizes_percent_and_unicode_when_expression_uses_symbols() {
        let result = try_build("10% + 2×3").expect("calculator result should be produced");
        assert_eq!("(10/100) + 2*3", result.normalized_expr);
        assert_eq!("6.1", result.formatted);
        assert_eq!("Six Point One", result.words);
    }

    #[test]
    fn test_try_build_labels_trigonometry_when_expression_calls_sin() {
        let result = try_build("sin(pi / 2)").expect("calculator result should be produced");
        assert_eq!("Sine", result.operation_name);
        assert_eq!("1", result.formatted);
    }

    #[test]
    fn test_format_value_uses_scientific_notation_when_magnitude_is_extreme() {
        assert_eq!(Some(String::from("1e20")), format_value(1e20));
        assert_eq!(Some(String::from("1e-10")), format_value(1e-10));
    }

    #[test]
    fn test_words_for_formatted_number_handles_negative_decimals_when_value_has_fraction() {
        assert_eq!(
            String::from("Negative Twelve Point Three Four"),
            words_for_formatted_number("-12.34")
        );
    }
}
