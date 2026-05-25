# Disk Space Cargo Cleanup

You are an unattended Claude Code maintenance session for Script Kit GPUI.

Goal: restore free disk space on this repo's volume. The trigger threshold is 25 GiB free; the target after cleanup is 35 GiB free or better.

Hard boundaries:
- Work only in the Script Kit GPUI repo.
- Do not edit source files.
- Do not use sudo.
- Do not touch `.git`.
- Do not delete anything outside `target/`, `target-agent/`, or the watcher state/log directories.
- You are authorized to terminate only Script Kit dev/Cargo processes using this repo's Cargo target dirs:
  - `./dev.sh`
  - `cargo watch` launched by `./dev.sh`
  - `scripts/agentic/agent-cargo.sh`
  - `cargo`, `cargo-watch`, and `rustc` processes whose cwd is this repo
  - PIDs listed in `target-agent/.locks/*.lock/pid`

Primary action:
Run the permitted helper command exactly as shown in the runtime section. That helper is intentionally scoped: it prunes Cargo targets, terminates only matching Script Kit dev/Cargo processes if needed, deletes bounded Cargo cache subdirectories only if pruning is insufficient, and verifies disk space afterward.

Expected flow:
1. Inspect the runtime facts.
2. Run the helper command.
3. Check `df -h .` and `du -sh target target-agent` afterward.
4. Return a concise summary of what changed and whether free disk is now above threshold.

Do not ask questions. Do not create plans or notes. Do not make commits.
