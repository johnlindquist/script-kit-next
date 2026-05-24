pub fn expanded_header_label(turn_count: usize) -> String {
    format!(
        "Cue - {turn_count} turn{}",
        if turn_count == 1 { "" } else { "s" }
    )
}
