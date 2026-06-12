mod window_state_audit {
    include!("tests/window_state.rs");
}

// NOTE: tests/persistence.rs is intentionally NOT included. It was a stale
// duplicate of src/window_state_persistence_tests.rs — running both in
// parallel races the process-global HOME env var (with_temp_state_dir) and
// makes a random subset fail every run. The live persistence suite is
// src/window_state_persistence_tests.rs.
