#!/usr/bin/env zsh
set -euo pipefail
source ~/.config/zsh/.zshrc 2>/dev/null || true

TASKS_DIR="./tasks"

# Build skill files list safely (no crash if missing)
skill_files=""
if [[ -d "./.opencode/skill" ]]; then
    skill_files=$(ls ./.opencode/skill/script* 2>/dev/null || echo "(no skill files found)")
else
    skill_files="(skill directory not found)"
fi

SUFFIX="

---

**Batch Context:** You are working through a series of automated tasks. Other prompts before you may have made changes to the codebase. Check the last few git commits to see what's changed since the batch started.

**Requirements:**

1. Use strict TDD
2. Commit often
3. In a research phase, read any skills files related to your task:

$skill_files

"

# Check if tasks directory exists
if [[ ! -d "$TASKS_DIR" ]]; then
    echo "No tasks directory found at $TASKS_DIR"
    exit 0
fi

# Get all .md files sorted in order (setopt handles empty glob)
setopt NULL_GLOB
task_files=("$TASKS_DIR"/*.md)
unsetopt NULL_GLOB

total=${#task_files[@]}

if [[ $total -eq 0 ]]; then
    echo "No task files found in $TASKS_DIR"
    exit 0
fi

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
        # Double quotes preserve content literally (no command execution)
        prompt="$(cat "$task_file")$SUFFIX"

        # Pass prompt via stdin for full agent capabilities
        echo "$prompt" | x --setting-sources "project,local" --verbose --print

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
