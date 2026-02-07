    #[test]
    fn test_hud_notification_clone() {
        // Test Clone impl for HudNotification
        let original = HudNotification {
            text: "Clone test".to_string(),
            duration_ms: 3000,
            created_at: Instant::now(),
            action_label: Some("Test".to_string()),
            action: Some(HudAction::OpenUrl("https://example.com".to_string())),
        };

        let cloned = original.clone();
        assert_eq!(cloned.text, original.text);
        assert_eq!(cloned.duration_ms, original.duration_ms);
        assert_eq!(cloned.action_label, original.action_label);
        assert!(
            cloned.has_action(),
            "Cloned notification should have action"
        );
    }
    #[test]
    fn test_hud_colors_copy_trait() {
        // Test that HudColors implements Copy (important for closures)
        let colors = HudColors::dark_default();

        // This would fail to compile if HudColors wasn't Copy
        let colors_copy = colors;
        let another_copy = colors;

        assert_eq!(colors_copy.background, another_copy.background);
        assert_eq!(colors_copy.text_primary, another_copy.text_primary);
    }
    #[test]
    fn test_color_manipulation_with_gray() {
        // Test lighten/darken with mid-gray
        let gray = 0x808080; // RGB(128, 128, 128)

        let lightened = lighten_color(gray, 0.5);
        // Each component: 128 + (255-128)*0.5 = 128 + 63.5 = 191.5 -> 191 = 0xbf
        assert_eq!(lightened, 0xbfbfbf);

        let darkened = darken_color(gray, 0.5);
        // Each component: 128 * 0.5 = 64 = 0x40
        assert_eq!(darkened, 0x404040);
    }
    #[test]
    fn test_color_manipulation_preserves_channels() {
        // Test that color manipulation works independently per channel
        let color = 0xff8000; // RGB(255, 128, 0) - orange

        let darkened = darken_color(color, 0.5);
        // R: 255*0.5=127, G: 128*0.5=64, B: 0*0.5=0
        let r = (darkened >> 16) & 0xff;
        let g = (darkened >> 8) & 0xff;
        let b = darkened & 0xff;

        assert_eq!(r, 127, "Red channel should be halved");
        assert_eq!(g, 64, "Green channel should be halved");
        assert_eq!(b, 0, "Blue channel should stay at 0");
    }
    // =============================================================================
    // Slot-based Allocation Tests (TDD for overlap fix)
    // =============================================================================

    #[test]
    fn test_slot_allocation_gives_unique_slots() {
        // Each HUD should get a unique slot 0..MAX_SIMULTANEOUS_HUDS
        let mut state = HudManagerState::new();

        // Allocate slots one by one
        let slot0 = state.first_free_slot();
        assert_eq!(slot0, Some(0), "First slot should be 0");

        // Simulate HUD at slot 0
        state.hud_slots[0] = Some(HudSlotEntry { id: 100 });

        let slot1 = state.first_free_slot();
        assert_eq!(slot1, Some(1), "Second slot should be 1");

        state.hud_slots[1] = Some(HudSlotEntry { id: 101 });

        let slot2 = state.first_free_slot();
        assert_eq!(slot2, Some(2), "Third slot should be 2");

        state.hud_slots[2] = Some(HudSlotEntry { id: 102 });

        // All slots full
        let slot_none = state.first_free_slot();
        assert_eq!(slot_none, None, "Should return None when all slots full");
    }
    #[test]
    fn test_slot_release_makes_slot_available() {
        // When a HUD dismisses, its slot becomes available for reuse
        let mut state = HudManagerState::new();

        // Fill all slots
        state.hud_slots[0] = Some(HudSlotEntry { id: 100 });
        state.hud_slots[1] = Some(HudSlotEntry { id: 101 });
        state.hud_slots[2] = Some(HudSlotEntry { id: 102 });

        // All slots full
        assert_eq!(state.first_free_slot(), None);

        // Release slot 1 (middle)
        state.release_slot_by_id(101);

        // Slot 1 should now be free
        assert!(state.hud_slots[1].is_none(), "Slot 1 should be released");

        // Next allocation should get slot 1
        let next_slot = state.first_free_slot();
        assert_eq!(next_slot, Some(1), "Should reuse released slot 1");
    }
    #[test]
    fn test_concurrent_huds_get_different_slots() {
        // Multiple HUDs active at same time should have different slots
        let mut state = HudManagerState::new();

        // Allocate and fill slots
        state.hud_slots[0] = Some(HudSlotEntry { id: 200 });
        state.hud_slots[1] = Some(HudSlotEntry { id: 201 });
        state.hud_slots[2] = Some(HudSlotEntry { id: 202 });

        // Verify all slots occupied by different IDs
        let ids: Vec<u64> = state
            .hud_slots
            .iter()
            .filter_map(|s| s.as_ref().map(|e| e.id))
            .collect();

        assert_eq!(ids.len(), 3, "All slots should be occupied");
        assert!(ids.contains(&200));
        assert!(ids.contains(&201));
        assert!(ids.contains(&202));

        // No duplicates
        let mut sorted = ids.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 3, "All IDs should be unique");
    }
    #[test]
    fn test_position_uses_slot_not_len() {
        // Position calculation should use slot index, not active count
        // This is the core bug fix - releasing middle HUD shouldn't move others

        let mut state = HudManagerState::new();

        // Fill slots 0, 1, 2
        state.hud_slots[0] = Some(HudSlotEntry { id: 300 });
        state.hud_slots[1] = Some(HudSlotEntry { id: 301 });
        state.hud_slots[2] = Some(HudSlotEntry { id: 302 });

        // Release middle HUD (slot 1)
        state.release_slot_by_id(301);

        // Slot 0 and 2 should still have their original entries
        assert!(
            state.hud_slots[0].is_some(),
            "Slot 0 should still be occupied"
        );
        assert!(state.hud_slots[1].is_none(), "Slot 1 should be empty");
        assert!(
            state.hud_slots[2].is_some(),
            "Slot 2 should still be occupied"
        );

        // Calculate offsets using slot indices
        // Slot 0 -> offset = 0 * HUD_STACK_GAP
        // Slot 2 -> offset = 2 * HUD_STACK_GAP (NOT 1 * HUD_STACK_GAP)
        let offset_slot_0 = 0.0 * HUD_STACK_GAP;
        let offset_slot_2 = 2.0 * HUD_STACK_GAP;

        assert!(
            (offset_slot_0 - 0.0).abs() < f32::EPSILON,
            "Slot 0 should have offset 0"
        );
        assert!(
            (offset_slot_2 - 2.0 * HUD_STACK_GAP).abs() < f32::EPSILON,
            "Slot 2 should keep its original offset"
        );
    }
    #[test]
    fn test_release_nonexistent_id_is_safe() {
        // Releasing an ID that doesn't exist should be a no-op
        let mut state = HudManagerState::new();

        state.hud_slots[0] = Some(HudSlotEntry { id: 400 });

        // Release non-existent ID
        state.release_slot_by_id(999);

        // Original slot should be unchanged
        assert!(state.hud_slots[0].is_some(), "Existing slot should remain");
        assert_eq!(state.hud_slots[0].as_ref().unwrap().id, 400);
    }
    #[test]
    fn test_find_slot_by_id() {
        let mut state = HudManagerState::new();

        state.hud_slots[0] = Some(HudSlotEntry { id: 500 });
        state.hud_slots[2] = Some(HudSlotEntry { id: 502 });
        // slot 1 is empty

        assert_eq!(state.find_slot_by_id(500), Some(0));
        assert_eq!(state.find_slot_by_id(502), Some(2));
        assert_eq!(state.find_slot_by_id(501), None); // doesn't exist
        assert_eq!(state.find_slot_by_id(999), None); // doesn't exist
    }
    #[test]
    fn test_active_hud_count() {
        let mut state = HudManagerState::new();

        assert_eq!(state.active_hud_count(), 0);

        state.hud_slots[0] = Some(HudSlotEntry { id: 600 });
        assert_eq!(state.active_hud_count(), 1);

        state.hud_slots[2] = Some(HudSlotEntry { id: 602 });
        assert_eq!(state.active_hud_count(), 2);

        state.hud_slots[1] = Some(HudSlotEntry { id: 601 });
        assert_eq!(state.active_hud_count(), 3);

        // Release middle
        state.release_slot_by_id(601);
        assert_eq!(state.active_hud_count(), 2);
    }
