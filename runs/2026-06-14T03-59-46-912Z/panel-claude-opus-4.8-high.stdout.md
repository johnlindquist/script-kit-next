You're right — I labeled three claims "unverified" and then reasoned on top of them. Let me actually check the source instead of leaving them as assertions.

I'll verify the three load-bearing claims that drove my recommendation: (1) the "AI runs CPU-only / slow" speed risk, (2) the "permissions onboarding incomplete" demo risk, (3) whether the semantic-ID/MCP transaction layer is real and how robust it looks.

Tooluse [Bash → "Verify local-llm + permissions + protocol claims"]: cd /Users/johnlindquist/dev/script-kit-gpui && echo "=== local-llm metal/CPU evidence ===" && rg -n "metal|CPU-only|cpu_only|ggml" src/ --type rust -i | rg -i "llama|whisper|metal|cpu" | head -30

=== local-llm metal/CPU evidence ===
