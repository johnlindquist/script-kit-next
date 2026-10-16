#[cfg(test)]
mod tests {
    use super::EmojiCategory::*;
    use super::*;

    #[test]
    fn the_dataset_is_the_full_unicode_set() {
        let all = parse_emoji_data(EMOJI_DATA);
        assert!(all.len() > 1900, "{}", all.len());
        assert!(EMOJIS.len() > 1800, "{}", EMOJIS.len());
        assert!(EMOJIS.iter().any(|emoji| emoji.emoji == "\u{1F600}"));
        assert!(
            all.iter()
                .filter(|emoji| emoji.supports_skin_tones())
                .count()
                > 300
        );
    }

    #[test]
    fn emoji_newer_than_the_mac_are_left_out() {
        assert_eq!(emoji_version_for_macos(14, 2), 150);
        assert_eq!(emoji_version_for_macos(14, 4), 151);
        assert_eq!(emoji_version_for_macos(15, 4), 160);
        let all = parse_emoji_data(EMOJI_DATA);
        let newest = all.iter().map(|emoji| emoji.version).max().unwrap();
        let supported = supported_emoji_version();
        assert!(EMOJIS.iter().all(|emoji| emoji.version <= supported));
        if newest > supported {
            assert!(EMOJIS.len() < all.len());
        }
    }

    #[test]
    fn skin_tones_apply_to_emoji_that_take_them() {
        let wave = emoji_by_value("\u{1F44B}").unwrap();
        assert_eq!(wave.with_tone(SkinTone::None), "\u{1F44B}");
        assert_eq!(wave.with_tone(SkinTone::Medium), "\u{1F44B}\u{1F3FD}");
        assert_eq!(
            emoji_for_variant("\u{1F44B}\u{1F3FF}").unwrap().emoji,
            "\u{1F44B}"
        );
        let rocket = emoji_by_value("\u{1F680}").unwrap();
        assert!(!rocket.supports_skin_tones());
        assert_eq!(rocket.with_tone(SkinTone::Dark), "\u{1F680}");
        for tone in SkinTone::iter() {
            assert_eq!(SkinTone::from_id(tone.id()), Some(tone));
        }
    }

    #[test]
    fn search_ranks_whole_words_and_names_first() {
        assert_eq!(search_emojis("cat")[0].emoji, "\u{1F408}");
        assert_eq!(search_emojis("thumbs up")[0].emoji, "\u{1F44D}");
        assert_eq!(search_emojis("rocket")[0].emoji, "\u{1F680}");
        // Every word must match: an unrelated second word leaves nothing.
        assert!(search_emojis("rocket zebra").is_empty());
        // Keywords match from the start of a word, never the middle
        // ("cat" doesn't find "location" or "dedicate").
        for emoji in search_emojis("cat") {
            let keyword_word = emoji.keywords.iter().any(|keyword| {
                keyword
                    .split_whitespace()
                    .any(|word| word.to_lowercase().starts_with("cat"))
            });
            assert!(keyword_word || emoji.name.contains("cat"), "{}", emoji.name);
        }
    }

    #[test]
    fn test_emoji_category_display_name_returns_human_readable_labels() {
        assert_eq!(SmileysEmotion.display_name(), "Smileys & Emotion");
        assert_eq!(PeopleBody.display_name(), "People & Body");
        assert_eq!(AnimalsNature.display_name(), "Animals & Nature");
        assert_eq!(FoodDrink.display_name(), "Food & Drink");
        assert_eq!(TravelPlaces.display_name(), "Travel & Places");
        assert_eq!(Activities.display_name(), "Activities");
        assert_eq!(Objects.display_name(), "Objects");
        assert_eq!(Symbols.display_name(), "Symbols");
        assert_eq!(Flags.display_name(), "Flags");
    }

    #[test]
    fn test_all_categories_has_expected_display_order() {
        let categories: Vec<EmojiCategory> = all_categories().collect();
        assert_eq!(
            categories,
            vec![
                SmileysEmotion,
                PeopleBody,
                AnimalsNature,
                FoodDrink,
                TravelPlaces,
                Activities,
                Objects,
                Symbols,
                Flags
            ]
        );
    }

    #[test]
    fn test_emojis_by_category_returns_only_requested_category() {
        let travel_emojis = emojis_by_category(TravelPlaces);
        assert!(!travel_emojis.is_empty());
        assert!(travel_emojis
            .iter()
            .all(|emoji| emoji.category == TravelPlaces));
    }

    #[test]
    fn test_grouped_emojis_returns_all_categories_in_display_order() {
        let grouped = grouped_emojis();
        assert_eq!(grouped.len(), all_categories().count());

        for ((category, emojis), expected_category) in grouped.iter().zip(all_categories()) {
            assert_eq!(*category, expected_category);
            assert!(emojis
                .iter()
                .all(|emoji| emoji.category == expected_category));
        }
    }

    #[test]
    fn test_grouped_emojis_covers_all_entries() {
        let grouped = grouped_emojis();
        let total_grouped_emojis: usize = grouped.iter().map(|(_, emojis)| emojis.len()).sum();
        assert_eq!(total_grouped_emojis, EMOJIS.len());
    }

    #[test]
    fn test_emoji_database_meets_category_targets() {
        assert!(emojis_by_category(SmileysEmotion).len() >= 50);
        assert!(emojis_by_category(PeopleBody).len() >= 30);
        assert!(emojis_by_category(AnimalsNature).len() >= 20);
        assert!(emojis_by_category(FoodDrink).len() >= 15);
        assert!(emojis_by_category(TravelPlaces).len() >= 15);
        assert!(emojis_by_category(Activities).len() >= 10);
        assert!(emojis_by_category(Objects).len() >= 15);
        assert!(emojis_by_category(Symbols).len() >= 15);
        assert!(emojis_by_category(Flags).len() >= 10);
    }

    #[test]
    fn test_search_emojis_matches_name_when_query_has_different_case() {
        let matches = search_emojis("GRINNING");
        assert!(matches.iter().any(|emoji| emoji.emoji == "😀"));
    }

    #[test]
    fn test_search_emojis_matches_keyword_when_query_is_substring() {
        let matches = search_emojis("appro");
        assert!(matches.iter().any(|emoji| emoji.emoji == "👍"));
    }

    #[test]
    fn test_search_emojis_returns_all_when_query_is_empty() {
        let matches = search_emojis("   ");
        assert_eq!(matches.len(), EMOJIS.len());
    }

    #[test]
    fn test_filtered_grid_row_count_matches_current_dataset() {
        let total = filtered_grid_row_count("", None, &EmojiHead::default());
        assert!(total > 0, "unfiltered grid should have rows");

        // "heart" filter should return a smaller count
        let heart = filtered_grid_row_count("heart", None, &EmojiHead::default());
        assert!(
            heart < total,
            "heart filter should have fewer rows than unfiltered"
        );
        assert!(heart > 0, "heart filter should have some rows");

        // "pizza" filter should be very small
        let pizza = filtered_grid_row_count("pizza", None, &EmojiHead::default());
        assert!(pizza > 0 && pizza <= 4, "pizza filter should have 1-4 rows");
    }

    #[test]
    fn emoji_grid_layout_moves_across_ragged_rows() {
        let layout = EmojiGridLayout {
            rows: vec![
                EmojiCellRow {
                    visible_row_index: 1,
                    start_index: 0,
                    count: 4,
                },
                EmojiCellRow {
                    visible_row_index: 2,
                    start_index: 4,
                    count: 1,
                },
                EmojiCellRow {
                    visible_row_index: 4,
                    start_index: 5,
                    count: 4,
                },
            ],
            item_to_row: vec![0, 0, 0, 0, 1, 2, 2, 2, 2],
        };

        // Down from full row into short row (column clamping: col 3 → col 0)
        assert_eq!(layout.move_index(3, EmojiNavDirection::Down), 4);
        // Down from short row into next full row
        assert_eq!(layout.move_index(4, EmojiNavDirection::Down), 5);
        // Up from full row into short row with column clamping (col 3 → col 0)
        assert_eq!(layout.move_index(8, EmojiNavDirection::Up), 4);
        // scroll_row_for_index returns correct visible_row_index
        assert_eq!(layout.scroll_row_for_index(5), 4);
        // Left wrapping to previous row's last item
        assert_eq!(
            layout.move_index(5, EmojiNavDirection::Left),
            4,
            "Left from first cell of row 2 should wrap to last cell of row 1"
        );
        // Right wrapping to next row's first item
        assert_eq!(
            layout.move_index(4, EmojiNavDirection::Right),
            5,
            "Right from last cell of short row should wrap to first cell of next row"
        );
    }

    #[test]
    fn emoji_grid_layout_build_produces_correct_rows() {
        // Build layout from real emoji data and verify structure
        let ordered = filtered_ordered_emojis("", None);
        let layout = build_emoji_grid_layout(&ordered, GRID_COLS, |e| e.category);

        // Every item should map to a valid row
        for (i, &row_ix) in layout.item_to_row.iter().enumerate() {
            assert!(
                row_ix < layout.rows.len(),
                "item {i} maps to out-of-bounds row {row_ix}"
            );
            let row = &layout.rows[row_ix];
            assert!(
                i >= row.start_index && i < row.start_index + row.count,
                "item {i} not within its mapped row (start={}, count={})",
                row.start_index,
                row.count
            );
        }
    }

    /// Locks the balanced emoji-grid rhythm: larger glyph-bearing tiles,
    /// enough columns to use the window width, and equal horizontal and
    /// vertical center-to-center spacing (`tile + gap == row height`).
    #[test]
    fn test_emoji_picker_grid_layout_constants_match_density_targets() {
        assert_eq!(GRID_COLS, 12);
        assert_eq!(GRID_TILE_SIZE, 48.0);
        assert_eq!(GRID_TILE_GAP, 8.0);
        assert_eq!(GRID_ROW_HEIGHT, 56.0);
        assert_eq!(GRID_TILE_SIZE + GRID_TILE_GAP, GRID_ROW_HEIGHT);
        assert_eq!(GRID_GLYPH_SCALE, 0.75);
        assert_eq!(GRID_TILE_SIZE * GRID_GLYPH_SCALE, 36.0);
    }

    #[test]
    fn test_grid_layout_scroll_rows_are_monotonic() {
        let ordered = filtered_ordered_emojis("", None);
        assert!(!ordered.is_empty(), "emoji dataset should not be empty");

        let layout = build_emoji_grid_layout(&ordered, GRID_COLS, |emoji| emoji.category);
        let mut last_row = layout.scroll_row_for_index(0);

        for ix in 1..ordered.len() {
            let row = layout.scroll_row_for_index(ix);
            assert!(
                row >= last_row,
                "scroll row must be monotonic: ix={} row={} previous_row={}",
                ix,
                row,
                last_row
            );
            last_row = row;
        }
    }

    #[test]
    fn test_grid_layout_left_and_right_clamp_at_edges() {
        let ordered = filtered_ordered_emojis("", None);
        assert!(!ordered.is_empty(), "emoji dataset should not be empty");

        let layout = build_emoji_grid_layout(&ordered, GRID_COLS, |emoji| emoji.category);
        assert_eq!(layout.move_index(0, EmojiNavDirection::Left), 0);

        let last = ordered.len() - 1;
        assert_eq!(layout.move_index(last, EmojiNavDirection::Right), last);
    }

    fn head(pinned: &[&str], frequent: &[&str]) -> EmojiHead {
        EmojiHead {
            pinned: pinned.iter().map(|value| value.to_string()).collect(),
            frequent: frequent.iter().map(|value| value.to_string()).collect(),
        }
    }

    #[test]
    fn browsing_shows_pinned_then_frequent_then_categories_without_repeats() {
        let display = display_ordered_emojis(
            "",
            None,
            &head(
                &["\u{1F680}", "\u{1F44B}\u{1F3FD}"],
                &["\u{1F680}", "\u{1F525}"],
            ),
        );
        let titles: Vec<&str> = display.sections.iter().map(|s| s.title.as_str()).collect();
        assert_eq!(
            &titles[..3],
            &[PINNED_LABEL, FREQUENTLY_USED_LABEL, "Smileys & Emotion"]
        );
        assert_eq!(display.pinned_count(), 2);
        // The rocket is pinned, so Frequently Used doesn't repeat it.
        assert_eq!(display.frequent_count(), 1);
        // A toned pin resolves to its base emoji.
        assert_eq!(display.emojis[1].emoji, "\u{1F44B}");
        let rockets = display
            .emojis
            .iter()
            .filter(|e| e.emoji == "\u{1F680}")
            .count();
        assert_eq!(rockets, 1);
        assert_eq!(display.emojis.len(), EMOJIS.len());
    }

    #[test]
    fn searching_shows_one_ranked_results_section() {
        let display = display_ordered_emojis("rocket", None, &head(&["\u{1F525}"], &[]));
        assert_eq!(display.sections.len(), 1);
        assert_eq!(display.sections[0].title, RESULTS_LABEL);
        assert_eq!(display.emojis[0].emoji, "\u{1F680}");
        let category = display_ordered_emojis("", Some(Flags), &head(&["\u{1F680}"], &[]));
        assert_eq!(category.sections.len(), 1);
        assert!(category.emojis.iter().all(|emoji| emoji.category == Flags));
    }

    #[test]
    fn display_grid_layout_keeps_head_sections_separate() {
        let display =
            display_ordered_emojis("", None, &head(&["\u{2764}\u{FE0F}"], &["\u{1F525}"]));
        let layout = build_display_grid_layout(&display, GRID_COLS);
        // Pinned header (0), pinned row (1), Frequently Used header (2), its row (3).
        assert_eq!(layout.rows[layout.item_to_row[0]].visible_row_index, 1);
        assert_eq!(layout.rows[layout.item_to_row[1]].visible_row_index, 3);
        assert!(layout.rows[layout.item_to_row[2]].visible_row_index >= 5);
        assert_eq!(
            display.row_count(GRID_COLS),
            layout.rows.last().unwrap().visible_row_index + 1
        );
        let down = layout.move_index(0, EmojiNavDirection::Down);
        assert_eq!(down, 1);
        assert_eq!(layout.move_index(down, EmojiNavDirection::Up), 0);
    }

    #[test]
    fn display_grid_layout_matches_build_emoji_grid_when_no_head() {
        let display = display_ordered_emojis("", None, &EmojiHead::default());
        assert_eq!(display.frequent_count(), 0);
        let base = filtered_ordered_emojis("", None);
        let legacy = build_emoji_grid_layout(&base, GRID_COLS, |e| e.category);
        let new = build_display_grid_layout(&display, GRID_COLS);
        assert_eq!(legacy.rows, new.rows);
        assert_eq!(legacy.item_to_row, new.item_to_row);
    }

    #[test]
    fn compute_display_scroll_row_matches_layout_visible_row_index() {
        for display in [
            display_ordered_emojis(
                "",
                None,
                &head(
                    &["\u{1F680}"],
                    &["\u{2764}\u{FE0F}", "\u{1F525}", "\u{1F389}"],
                ),
            ),
            display_ordered_emojis("heart", None, &EmojiHead::default()),
        ] {
            let layout = build_display_grid_layout(&display, GRID_COLS);
            for ix in 0..display.emojis.len() {
                assert_eq!(
                    compute_display_scroll_row(ix, &display),
                    layout.scroll_row_for_index(ix),
                    "ix={ix}"
                );
            }
        }
    }

    #[test]
    fn test_grid_layout_down_then_up_round_trips_when_destination_exists() {
        let ordered = filtered_ordered_emojis("", None);
        assert!(
            ordered.len() > GRID_COLS,
            "need at least two rows of emoji data for navigation coverage"
        );

        let layout = build_emoji_grid_layout(&ordered, GRID_COLS, |emoji| emoji.category);

        let mut found_round_trip = false;
        for ix in 0..(ordered.len() - GRID_COLS) {
            let down = layout.move_index(ix, EmojiNavDirection::Down);
            if down != ix {
                found_round_trip = true;
                assert_eq!(
                    layout.move_index(down, EmojiNavDirection::Up),
                    ix,
                    "down/up should round-trip when a destination cell exists: start={} down={}",
                    ix,
                    down
                );
                break;
            }
        }

        assert!(
            found_round_trip,
            "expected at least one index with a valid downward move"
        );
    }

    #[test]
    fn shortcodes_turn_into_emoji_as_you_type_the_closing_colon() {
        assert_eq!(emoji_for_shortcode("rocket"), Some("\u{1F680}"));
        assert_eq!(emoji_for_shortcode("red_heart"), Some("\u{2764}\u{FE0F}"));
        assert_eq!(emoji_for_shortcode("+1"), Some("\u{1F44D}"));
        assert_eq!(emoji_for_shortcode("Tada"), Some("\u{1F389}"));
        assert_eq!(
            emoji_for_shortcode("flag_japan"),
            Some("\u{1F1EF}\u{1F1F5}")
        );
        assert_eq!(emoji_for_shortcode("not_an_emoji"), None);
        assert_eq!(shortcode_for_name("flag: Japan"), "flag_japan");

        // The cursor sits right after the typed character.
        let typed = |previous: &str, content: &str| {
            let at = previous
                .bytes()
                .zip(content.bytes())
                .position(|(a, b)| a != b)
                .unwrap_or(previous.len());
            shortcode_typed_fixup(previous, content, at + 1)
        };
        assert_eq!(
            typed("launch :rocket", "launch :rocket:"),
            Some(("launch \u{1F680}".to_string(), 11))
        );
        assert_eq!(
            typed(":tada done", ":tada: done"),
            Some(("\u{1F389} done".to_string(), 4))
        );
        // Times, URLs, and unknown names stay as typed.
        assert_eq!(typed("at 10:30", "at 10:30:"), None);
        assert_eq!(typed("see http:rocket", "see http:rocket:"), None);
        assert_eq!(typed("hi :nope", "hi :nope:"), None);
        assert_eq!(typed("a :two words", "a :two words:"), None);
        // Pasting more than one character never triggers it.
        assert_eq!(shortcode_typed_fixup("x ", "x :rocket:", 10), None);
    }
}
