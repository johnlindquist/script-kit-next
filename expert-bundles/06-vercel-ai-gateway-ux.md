# Vercel AI Gateway UX Expert Bundle

## Original Goal

> Adding a feature where there's a simple UX, new user experience for adding a key/oidc from the vercel ai gateway
>
> This is the original task description that prompted the creation of this bundle.

## Executive Summary

Script Kit's AI window currently supports multiple providers (Anthropic, OpenAI) via environment variables. Adding Vercel AI Gateway support requires a user-friendly setup flow for API keys and OIDC authentication.

### Key Problems:
1. **No guided setup** - Users must manually set environment variables
2. **OIDC flow missing** - Vercel's OAuth flow not implemented
3. **Key validation** - No way to test if API key is valid before saving

### Required Fixes:
1. **src/ai/config.rs** - Add Vercel AI Gateway configuration
2. **src/ai/providers.rs** - Implement Vercel provider with gateway URL
3. **src/ai/window.rs** - Add setup wizard UI for first-time users
4. **New: src/ai/vercel_auth.rs** - OIDC authentication flow

### Files Included:
- `src/ai/config.rs`: AI provider configuration and key detection
- `src/ai/providers.rs`: Provider implementations (OpenAI, Anthropic)
- `src/ai/window.rs`: AI chat window UI
- `src/ai/model.rs`: Chat and message data models
- `src/ai/mod.rs`: Module exports

---

