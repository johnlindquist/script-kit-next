# Getting Started with Script Kit GPUI

Use the Mac app to run a TypeScript file from the launcher. You don’t need a source checkout, Rust, or an AI account for this.

## Start with the Mac app

Download the Mac app ZIP from [GitHub Releases](https://github.com/johnlindquist/script-kit-next/releases), not one of GitHub’s source archives. Unzip it, move **Script Kit.app** to Applications, and open it from Finder. Install [Bun](https://bun.sh) to run scripts.

<a id="3-your-first-script"></a>

## Run your first script

1. Press **⌘;**, search for **New Script**, and choose the row for a blank script (not **New Script from Template**).
2. Name it **Hello Kit** and press Return. Replace the starter code in the new file with the following, then save it:

```typescript
import "@scriptkit/sdk";

export const metadata = { name: "Hello Kit" };

const who = await arg("Who would you like to greet?", ["Kit", "World"]);
await div(who === "World" ? "Hello, world!" : "Hello, Kit!");
```

Press **⌘;**, search for **Hello Kit**, and run it. Choose **World** to see **Hello, world!** Run it again and choose **Kit** to see **Hello, Kit!**

**Where is the file?** New Script uses the workspace Kit has open. Your scripts folder is `plugins/main/scripts/` inside it: `~/.scriptkit/plugins/main/scripts/` by default, or `<SK_PATH>/plugins/main/scripts/` if you configured another workspace. If the editor didn’t open, open the newly created `.ts` file there. If you create a script by hand, choose an unused filename in that same folder.

If you set `SK_PATH`, quote the entire folder name, including any spaces. For scripts and settings commands, use an absolute path without `..` (a direct symlink is fine). From inside the intended folder, `pwd -P` shows its physical path. To use the default again, run `unset SK_PATH` rather than setting it to an empty string. Kit’s settings helper, CLI and SDK then use `.scriptkit` under your actual home directory, without interpreting dollar signs in its name. If you supply a custom `HOME`, give its absolute path without `..`. The settings SDK also checks that home before looking for `~/.bun/bin/bun` under Node; a fully explicit `BUN_PATH` and `SK_PATH` do not depend on that lookup.

### If Kit can't find Bun

Kit needs [Bun](https://bun.sh) to run TypeScript scripts. Opening an app from Finder does not run your shell startup files. Kit checks common user and Homebrew install locations, absolute PATH entries outside the working directory, and direct Bun installations in mise and asdf (including their data-directory overrides when available to the app). It does not run either manager or let a project-local shim pick the binary.

If you use mise or asdf, run `mise which bun` or `asdf which bun` in a trusted directory where the version you want is active. Use the absolute filename it prints, not the `shims/bun` path that `command -v bun` may return. A link in `/usr/local/bin` is fine if it points to the installed Bun executable. A link that goes through `shims/bun` or `node_modules/.bin/bun` still leads through a shim, so Kit skips it during discovery and refuses it as an explicit pin. If Kit sees more than one managed binary, or yours lives elsewhere, put that filename in your existing `~/.scriptkit/config.ts` (or the `config.ts` under your chosen `SK_PATH`), keeping the rest of your config:

```ts
// Inside export default { ... }
bun_path: "/Users/you/.local/share/mise/installs/bun/1.3.14/bin/bun",
```

For asdf, a typical path is `~/.asdf/installs/bun/<version>/bin/bun`. Substitute your full home path and installed version; don't use `~`, a `shims` path, or `./bun`. A computed `bun_path` requires another discoverable Bun to evaluate `config.ts` on cold start. Relaunch from Finder and run a simple script to confirm. If a pinned version is removed, update the pin; script execution will not silently switch binaries.

`bun_path` tells the Mac app what to run; it does not change a terminal or an external agent’s `PATH`. To read Kit’s offline CLI manual in an environment without Bun on `PATH`, invoke the resolved Bun directly with Kit’s startup options (adjust the workspace path if you use `SK_PATH`). This keeps the current project’s Bun preloads and `.env` files out of the command without changing how Bun runs your own scripts:

```sh
"/absolute/path/to/bun" --config=/dev/null --no-env-file --no-install --no-macros \
  "$HOME/.scriptkit/bin/scriptkit" --skill
```

If a Node script imports Kit’s settings SDK (`sdk/kit-sdk-config.ts`), it can select Bun with the **`BUN_PATH` environment variable**. Use the absolute executable path from your manager, not a `shims` or `node_modules/.bin` entry. An invalid pin stops the settings request. Without a pin, the SDK looks in the usual install locations and then on `PATH`, skipping project-local candidates in the working directory. Keep the rest of `PATH` trusted. A script already running in Bun reuses that Bun.

To change one setting safely, read its key and pass the returned `revision` to `config.set(key, value, revision)` or `config.reset(key, revision)`. If your key or revision variable is `undefined`, the call refuses rather than reading everything, resetting everything, or saving without a check. For an intentional whole-config reset, review `await config.get()` first and pass its revision to `config.resetAll(revision)`. If the file contains manual-only or unsupported settings, a whole reset refuses without changing it. The no-argument `config.reset()` still requests a whole reset without a revision check.

When a settings write fails with `stale_revision` (`error.code` in the SDK; `errorCode` in the CLI’s JSON), someone has changed the file since you read it. Read it again and review the proposed edit before saving. If you see `settings_write_displaced`, the helper stopped before publishing because its lock or draft changed during the edit. Let the active writer finish, then reread your settings. Don’t remove a lock while another edit is in progress. If a draft remains in the lock after a writer has stopped, inspect it rather than deleting it blindly. An unreadable reply is not proof of a save.

If `config.ts` ends with a type annotation, start `as` or `satisfies` on the same line as the closing brace: `} satisfies Config;`. On a new line, those words no longer annotate the object. The settings tools leave that file unchanged; move the annotation back before editing through the SDK.

### Upgrading from a v1 kit

Automatic migration runs only for the default home and a recognized legacy layout. If `~/.kenv` also contains a Git repository, subkits, links, or other unrecognized content, Kit leaves that workspace in place. Keep your old files and use **Migrate v1 Scripts** to review what to bring across. Choosing another `SK_PATH` does not migrate the default home.

<a id="2-the-main-hotkey"></a>

## The main hotkey

The default launcher hotkey is `⌘;` (Cmd+Semicolon). To change it, edit the `hotkey` field in your active workspace’s `config.ts`. Keep your other settings. The default entry is:

```typescript
// Inside export default { ... }
hotkey: { modifiers: ["meta"], key: "Semicolon" },
```

The main hotkey understands more than one gesture:

| Gesture | Result |
| --- | --- |
| Tap | Open the launcher on release; hide it if Kit already has focus |
| Hold (~250 ms) | Dictate into the original field; release to finish |
| Double-tap | Open Today; hold the second press to dictate into it |

Use ⌘↵ in the empty launcher for a clean chat. See the [gesture guide](../specs/gesture-grammar.md) for routing and cancellation.

<a id="4-the-core-workflow"></a>

## The core workflow

1. **Hotkey** — press `⌘;` from anywhere.
2. **Say it** — type a script name, built-in, app, or a sentence saying what should happen. Search is local. With [remote readings enabled](jev-intent.md), Jev lists up to three readings of a sentence under a "Jev · ⌘1–3" header.
3. **Run** — press `Enter`. The ↵ label always names exactly what it will run.

Along the way:

- `⌘K` stages the selected row in the line above the input, and the list becomes its actions (edit, logs, copy path, and more). `Esc` lets go.
- `⌘I` toggles an info panel about the selected item.
- `Tab` on typed text hands the words to an agent (Tab again takes them back); `Tab` on an empty input cycles the available subject tabs. Type `>` to pick a working directory.
- Nothing asks "Are you sure?": something that can't be undone waits 8 s in the line ("Runs in 8s") and `⌘Z` stops it; something that can be undone just happens, and `⌘Z` takes it back.
- If nothing matches, fallback rows appear: **Do in Current App**, **Search Files**, **Ask AI**, open-as-URL/file, and an inline calculator for math input.

### Keep a built-in out of the main menu

You can use personal scripts without setting up Notes, window management, or an agent. The built-ins come with the app; there’s no separate scripts-only installer or first-run plugin chooser. For example, to hide the **Search Notes** command row, add `"builtin/search-notes"` to `hiddenCommands` in your workspace’s `config.ts`. If there’s already a list, keep its other entries. Remove just that ID to undo the change. This does not hide your notes, disable Notes, or revoke any permissions.

<a id="5-use-bun-packages"></a>

## Use Bun packages

Kit supplies `@scriptkit/sdk` in the workspace; you don’t need to install it from npm. If another script imports a third-party package, open a terminal in the workspace that contains its `plugins` folder and install just that package:

```bash
bun add zod
```

Then `import` them normally inside scripts. See [SDK Scripting](./sdk-scripting.md) for the full prompt API and metadata reference.

<a id="6-pick-your-next-guide"></a>

## Pick your next guide

- Big picture and hidden features: [Feature Tour](./feature-tour.md)
- Every launcher input mode and sigil: [Main Menu Input](./main-menu-input.md)
- Writing real scripts: [SDK Scripting](./sdk-scripting.md)
- Agents, context, and MCP: [MCP and Agent Context](./mcp-and-agent-context.md)
- Voice input: [Dictation](./dictation.md)
- Safe window observation: [Computer Use](./computer-use.md)

## Troubleshooting

| Symptom | Fix |
| --- | --- |
| Launcher doesn't open | Confirm the app is running and check `hotkey` in the active workspace’s `config.ts`. |
| Script doesn't appear | Check that the `.ts` file was saved in the active workspace’s `plugins/main/scripts/`, then choose **Reload Scripts** from Kit’s menu bar menu. |
| Imports fail at runtime | Run `bun add <package>` in the workspace that contains the script’s `plugins` folder. |
| MCP/agent examples fail | Start Script Kit and use the live `server.json` from that same workspace. |
| Dictation unavailable | Search **Dictation Setup** in the launcher and check model, microphone permission, and device readiness. |
| Snippets don't expand, or text is copied instead of pasted | Search **Set Up Permissions** and turn on Accessibility. See [Permissions](./permissions.md). |

<a id="prerequisites-for-building-from-source"></a>
<a id="1-build-and-run"></a>

## Build from source (contributors)

To develop the Mac app itself, install the [pinned Rust toolchain](../../rust-toolchain.toml) and [Bun](https://bun.sh), then run:

```bash
git clone https://github.com/johnlindquist/script-kit-next.git script-kit-gpui
cd script-kit-gpui
cargo build --release
./target/release/script-kit-gpui
```

For hot reload, install `cargo-watch` with `cargo install cargo-watch` and run `./dev.sh` in the checkout. These steps are not needed to write or run personal scripts.
