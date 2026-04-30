# Share a `LAT_LLM_KEY_HELPER` Cache Across Terminals Without Plaintext Secrets

If your `lat` setup looks like this:

```zsh
export LAT_LLM_KEY_HELPER='/opt/homebrew/bin/op item get LAT_LLM_KEY --vault Personal --fields credential --reveal'
```

then every `npx lat search` can trigger another 1Password round trip.

I replaced that with a wrapper:

```zsh
export LAT_LLM_KEY_FETCH_CMD='/opt/homebrew/bin/op item get LAT_LLM_KEY --vault Personal --fields credential --reveal'
export LAT_LLM_KEY_HELPER='/Users/johnlindquist/dev/script-kit-gpui/scripts/lat-llm-key-helper.sh'
unset LAT_LLM_KEY
```

What the wrapper does:

- keeps the `LAT_LLM_KEY_HELPER` contract unchanged
- fetches once from 1Password
- shares the cached key across terminal sessions
- uses the macOS login Keychain by default, so the secret is not written to plaintext cache on disk
- falls back to a file cache only when Keychain is unavailable or you force `LAT_LLM_KEY_CACHE_BACKEND=file`

Useful controls:

```zsh
export LAT_LLM_KEY_CACHE_TTL_SECONDS=43200
export LAT_LLM_KEY_KEYCHAIN_SERVICE='lat.llm-key'
```

Clear the cache:

```zsh
/Users/johnlindquist/dev/script-kit-gpui/scripts/lat-llm-key-helper.sh --clear
```

This ended up being a better fit than relying on 1Password CLI auth reuse, because the thing I actually wanted to cache globally was the fetched `LAT_LLM_KEY`, not the per-terminal CLI approval state.
