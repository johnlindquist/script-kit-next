#!/usr/bin/env zsh
set -euo pipefail
source ~/.config/zsh/.zshrc 2>/dev/null || true

TASKS_DIR="./tasks"
SUFFIX="

---

**Batch Context:** You are working through a series of automated tasks. Other prompts before you may have made changes to the codebase. Check the last few git commits to see what's changed since the batch started.

**Requirements:** Use strict TDD. Commit often."

# Get all .md files sorted in order
task_files=("$TASKS_DIR"/*.md)
total=${#task_files[@]}
current=0

echo "Found $total task(s) in $TASKS_DIR"
echo ""

for task_file in "${task_files[@]}"; do
    if [[ -f "$task_file" ]]; then
        current=$((current + 1))

        echo "=========================================="
        echo "Starting on prompt $current of $total: $(basename "$task_file")"
        echo "=========================================="

        # Read the prompt and append suffix
        prompt="$(cat "$task_file")$SUFFIX"

        # Run x with the prompt
        x -p "$prompt"

        # Delete the prompt file after completion
        rm "$task_file"

        echo ""
        echo "✓ Completed and deleted: $(basename "$task_file") ($current/$total done)"
        echo ""
    fi
done

echo "=========================================="
echo "All $total tasks completed."
echo "=========================================="
