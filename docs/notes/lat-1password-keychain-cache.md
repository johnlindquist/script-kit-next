# Caching `LAT_LLM_KEY_HELPER` Without Leaving the Key in Plaintext

`lat search` is useful, but wiring `LAT_LLM_KEY_HELPER` straight to `op item get ... --reveal` means every lookup can bounce through 1Password again. That gets old fast.

I wanted a setup with three properties:

1. `lat` still calls a helper command, so the upstream interface stays the same.
2. The fetched key is shared across terminal sessions.
3. The cached secret does not live as plaintext in `~/.cache` on macOS.

The result is a tiny wrapper script around the 1Password fetch command:

- `LAT_LLM_KEY_FETCH_CMD` holds the real fetch command.
- `LAT_LLM_KEY_HELPER` points at the wrapper.
- On macOS, the wrapper caches the key in the login Keychain.
- On other platforms, or when explicitly requested, it falls back to a file cache.
- A TTL controls when the helper goes back to 1Password.

In this repo that helper lives at `scripts/lat-llm-key-helper.sh`.

## Why not rely on 1Password CLI auth caching?

Because that cache is scoped to 1Password's CLI authorization model, not to your `lat` use case.

The friction points are:

- authorization is tied to terminal sessions
- new tabs and windows can trigger a fresh approval
- app locks invalidate the session
- the helper still executes `op ...` for every `lat search`

So the right place to smooth this out is one layer up: cache the fetched `LAT_LLM_KEY`, not just the CLI auth.

## The setup

The fetch command stays the same:

```zsh
export LAT_LLM_KEY_FETCH_CMD='/opt/homebrew/bin/op item get LAT_LLM_KEY --vault Personal --fields credential --reveal'
```

Then point `lat` at the wrapper:

```zsh
export LAT_LLM_KEY_HELPER='/Users/johnlindquist/dev/script-kit-gpui/scripts/lat-llm-key-helper.sh'
unset LAT_LLM_KEY
```

Now the first `npx lat search "..."` fetches from 1Password, stores the result in Keychain, and returns it. Later calls from any terminal session reuse the cached value until the TTL expires or the cache is cleared.

## How the helper works

The wrapper has two backends:

### 1. Keychain backend on macOS

When `security` is available, the helper stores the secret in the login Keychain under a service name like `lat.llm-key`. The cache metadata stays on disk, but the key does not.

That gives you:

- shared access across shells
- no plaintext key in a cache file
- a simple way to rotate or clear the cache

### 2. File backend elsewhere

If Keychain is unavailable, or if you explicitly set:

```zsh
export LAT_LLM_KEY_CACHE_BACKEND=file
```

the helper writes the secret to a private cache file and protects it with `umask 077`.

That is less desirable on macOS, but still useful as a portable fallback.

## Cache invalidation

The helper invalidates the cache when:

- the TTL expires
- the fetch command changes
- you run `scripts/lat-llm-key-helper.sh --clear`

The fetch-command hash matters because it avoids subtle stale-cache behavior when you switch vaults, item names, or retrieval logic.

## Useful knobs

```zsh
export LAT_LLM_KEY_CACHE_TTL_SECONDS=43200
export LAT_LLM_KEY_KEYCHAIN_SERVICE='lat.llm-key'
```

- `LAT_LLM_KEY_CACHE_TTL_SECONDS` sets cache lifetime in seconds.
- `LAT_LLM_KEY_KEYCHAIN_SERVICE` changes the Keychain service name.
- `LAT_LLM_KEY_CACHE_BACKEND=file` forces file caching.
- `LAT_LLM_KEY_CACHE_FILE=/path/to/cache` moves the file backend.

To clear the shared cache:

```zsh
/Users/johnlindquist/dev/script-kit-gpui/scripts/lat-llm-key-helper.sh --clear
```

## The practical payoff

The nice part is that nothing in `lat` needs to change. It still calls a helper command. The helper just got smarter:

- one 1Password fetch
- reuse across terminals
- Keychain-backed secret storage on macOS
- explicit invalidation when you want it

That is a much better fit for day-to-day `lat search` use than paying the full 1Password round trip every time.
