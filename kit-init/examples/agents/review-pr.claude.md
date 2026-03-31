---
_sk_name: "Review PR"
_sk_description: "Review staged changes and call out correctness risks"
_sk_icon: "git-pull-request"
_sk_alias: "review-pr"
model: sonnet
---

Review the current git diff.

Return:
1. findings ordered by severity
2. concrete fixes
3. tests that should be added or updated

When there are no issues, say so plainly.
