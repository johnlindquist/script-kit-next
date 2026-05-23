---
title: 2026-05-22-17-50-find-source-of-all-ten-skill
type: note
permalink: script-kit-gpui/conversations/2026-05-22-17-50-find-source-of-all-ten-skill
tags:
- codex-conversation
---
# Find Source of All-Ten Skill

Started: 2026-05-22 17:50 UTC
Repo: script-kit-gpui

## Round 1

### You

$beast-academy---all-ten <--- where is this skill coming from?

### Tools

#### Tool 1 — Shell

```sh
git status --short --untracked-files=all
```

Result: completed, first 8 of 52 lines shown

```text
M src/actions/dialog.rs
M src/app_execute/builtin_execution.rs
M src/app_impl/filter_input_change.rs
M src/app_impl/filter_input_core.rs
M src/app_impl/filter_input_updates.rs
M src/app_impl/lifecycle_reset.rs
M src/app_impl/menu_syntax_main_hint.rs
M src/app_impl/registries_state.rs
... truncated, 44 more lines omitted
```

#### Tool 2 — Shell

```sh
rg -n "beast-academy|all-ten|beast-academy---all-ten" . /Users/johnlindquist/.codex /Users/johnlindquist/.agents 2>/dev/null
```

Result: completed, first 2 of 2 lines shown

```text
/Users/johnlindquist/.codex/log/codex-tui.log:3463184:2026-05-22T17:50:37.099166Z  INFO session_loop{thread_id=019e50ce-54c5-7422-b852-060ca93a114b}:submission_dispatch{otel.name="op.dispatch.user_inp
... truncated ...
/Users/johnlindquist/.codex/log/codex-tui.log:3463185:2026-05-22T17:50:37.105430Z  INFO session_loop{thread_id=019e50ce-54c5-7422-b852-060ca93a114b}:submission_dispatch{otel.name="op.dispatch.user_inp
... truncated ...
```

#### Tool 3 — Shell

```sh
find . /Users/johnlindquist/.codex /Users/johnlindquist/.agents -path '*beast*' -o -path '*all-ten*' 2>/dev/null | head -200
```

Result: completed, 1 line

```text
./memories/conversations/2026-05-22-17-50-find-source-of-all-ten-skill.md
```

#### Tool 4 — Shell

```sh
sed -n '5188,5224p' /Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json
```

Result: completed, first 8 of 37 lines shown

```text
"status": "UNSET"
},
"categories": [
"EDUCATION"
],
"subCategories": null,
"seoDescription": null,
"screenshots": null,
... truncated, 29 more lines omitted
```

#### Tool 5 — Shell

```sh
rg -n "asdk_app_69a66c553c248191b2b2bc556373335b|beast-academy---all-ten|beast academy|All Ten" /Users/johnlindquist/.codex/cache /Users/johnlindquist/.codex/config* /Users/johnlindquist/.codex/plugins /Users/johnlindquist/.agents 2>/dev/null
```

Result: completed, first 8 of 15 lines shown

```text
/Users/johnlindquist/.codex/plugins/data/toolsmith-better-plugins/indexes/live-index.json:207:    "rg -n \"<str>\" . /<path> /<path> <num>>/dev/null": "rg -n \"beast-academy|all-ten|beast-academy---al
... truncated ...
/Users/johnlindquist/.codex/plugins/data/toolsmith-better-plugins/indexes/live-index.json:210:    "rg -n \"<str>\" /<path> /<path> /<path> /<path>": "rg -n \"asdk_app_69a66c553c248191b2b2bc556373335b|
... truncated ...
/Users/johnlindquist/.codex/plugins/data/toolsmith-better-plugins/indexes/live-index.json:219:      "excerpt": "$beast-academy---all-ten <--- where is this skill coming from?",
/Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json:5179:      "id": "asdk_app_69a66c553c248191b2b2bc556373335b",
/Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json:5180:      "name": "Beast Academy - All Ten",
/Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json:5182:      "logoUrl": "connectors://asdk_app_69a66c553c248191b2b2bc556373335b/logo?theme=light",
... truncated, 9 more lines omitted
```

#### Tool 6 — Shell

```sh
sed -n '5176,5212p' /Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json
```

Result: completed, first 8 of 37 lines shown

```text
"pluginDisplayNames": []
},
{
"id": "asdk_app_69a66c553c248191b2b2bc556373335b",
"name": "Beast Academy - All Ten",
"description": "Play today's number puzzle",
"logoUrl": "connectors://asdk_app_69a66c553c248191b2b2bc556373335b/logo?theme=light",
"logoUrlDark": null,
... truncated, 29 more lines omitted
```

#### Tool 7 — Shell

```sh
nl -ba /Users/johnlindquist/.codex/history.jsonl | sed -n '2329p'
```

Result: completed, 1 line

```text
2329	{"session_id":"019e50ce-54c5-7422-b852-060ca93a114b","ts":1779472229,"text":"[$beast-academy---all-ten](app://asdk_app_69a66c553c248191b2b2bc556373335b) <--- where is this skill coming from?"}
```

#### Tool 8 — Shell

```sh
nl -ba /Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json | sed -n '5178,5210p'
```

Result: completed, first 8 of 33 lines shown

```text
5178	    {
5179	      "id": "asdk_app_69a66c553c248191b2b2bc556373335b",
5180	      "name": "Beast Academy - All Ten",
5181	      "description": "Play today's number puzzle",
5182	      "logoUrl": "connectors://asdk_app_69a66c553c248191b2b2bc556373335b/logo?theme=light",
5183	      "logoUrlDark": null,
5184	      "distributionChannel": "ECOSYSTEM_DIRECTORY",
5185	      "branding": null,
... truncated, 25 more lines omitted
```

### Assistant

It’s not a Codex skill. It’s coming from the Codex Apps/Connectors directory cache.

Source found:

- App directory cache: [/Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json:5179](/Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json:5179)
  - `id`: `asdk_app_69a66c553c248191b2b2bc556373335b`
  - `name`: `Beast Academy - All Ten`
  - `description`: `Play today's number puzzle`
  - `distributionChannel`: `ECOSYSTEM_DIRECTORY`
  - `installUrl`: `https://chatgpt.com/apps/beast-academy---all-ten/asdk_app_69a66c553c248191b2b2bc556373335b`
  - `isAccessible`: `false`
  - `isEnabled`: `true`

Your prompt history confirms Codex rendered it as an app link, not a skill trigger:

[/Users/johnlindquist/.codex/history.jsonl:2329](/Users/johnlindquist/.codex/history.jsonl:2329)

```text
[$beast-academy---all-ten](app://asdk_app_69a66c553c248191b2b2bc556373335b)
```

So `$beast-academy---all-ten` is an app connector slug from the ecosystem directory cache, not something in `.agents/skills`, `.codex/skills`, or this repo’s skill map.

## Round 2

### You

How did this get added? Can you see when/what time those files changed and help me track down what happened?

### Tools

#### Tool 1 — Shell

```sh
stat -f '%N%n  birth:  %SB%n  modify: %Sm%n  change: %Sc%n  access: %Sa%n  size:   %z' /Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json /Users/johnlindquist/.codex/history.jsonl /Users/johnlindquist/.codex/log/codex-tui.log
```

Result: completed, first 8 of 18 lines shown

```text
/Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json
birth:  May 14 15:00:50 2026
modify: May 22 11:49:33 2026
change: May 22 11:49:33 2026
access: May 22 11:50:50 2026
size:   1287057
/Users/johnlindquist/.codex/history.jsonl
birth:  May 16 21:32:32 2026
... truncated, 10 more lines omitted
```

#### Tool 2 — Shell

```sh
ls -lT /Users/johnlindquist/.codex/cache/codex_app_directory /Users/johnlindquist/.codex/history.jsonl /Users/johnlindquist/.codex/log/codex-tui.log
```

Result: completed, 6 lines

```text
-rw-------  1 johnlindquist  staff      2315191 May 22 11:51:40 2026 /Users/johnlindquist/.codex/history.jsonl
-rw-------  1 johnlindquist  staff  15539031785 May 22 11:51:54 2026 /Users/johnlindquist/.codex/log/codex-tui.log

/Users/johnlindquist/.codex/cache/codex_app_directory:
total 2520
-rw-r--r--  1 johnlindquist  staff  1287057 May 22 11:49:33 2026 b7d8cf0866ab0da129592cf7a860d948c75ddd88.json
```

#### Tool 3 — Shell

```sh
find /Users/johnlindquist/.codex -iname '*app*directory*' -o -iname '*connect*' -o -iname '*asdk*' 2>/dev/null | head -200
```

Result: completed, first 8 of 20 lines shown

```text
/Users/johnlindquist/.codex/vendor_imports/skills/skills/.curated/figma-generate-library/references/code-connect-setup.md
/Users/johnlindquist/.codex/vendor_imports/skills/skills/.curated/cloudflare-deploy/references/network-interconnect
/Users/johnlindquist/.codex/vendor_imports/skills/skills/.curated/figma-code-connect-components
/Users/johnlindquist/.codex/cache/codex_app_directory
/Users/johnlindquist/.codex/.tmp/plugins/plugins/build-web-apps/skills/stripe-best-practices/references/connect.md
/Users/johnlindquist/.codex/.tmp/plugins/plugins/cloudflare/skills/cloudflare/references/network-interconnect
/Users/johnlindquist/.codex/.tmp/plugins/plugins/twilio-developer-kit/skills/twilio-agent-connect
/Users/johnlindquist/.codex/.tmp/plugins/plugins/figma/agents/figma-code-connect-agent.md
... truncated, 12 more lines omitted
```

#### Tool 4 — Shell

```sh
rg -n "app directory|codex_app_directory|asdk_app_69a66c553c248191b2b2bc556373335b|Beast Academy|beast-academy" /Users/johnlindquist/.codex/memories/MEMORY.md /Users/johnlindquist/.codex/log/codex-tui.log /Users/johnlindquist/.codex/history.jsonl 2>/dev/null
```

Result: completed, first 8 of 15 lines shown

```text
Total output lines: 17

/Users/johnlindquist/.codex/log/codex-tui.log:2074823:2026-05-15T18:45:15.528224Z  INFO turn{otel.name="session_task.turn" thread.id=019e2ba9-07d8-7810-b928-d03bb2505578 turn.id=a42d3289-4c5d-4ec7-a5c
... truncated ...
/Users/johnlindquist/.codex/log/codex-tui.log:2075124:+        ["drag payload resolves the host .app directory and writes a fileURL pasteboard item"]
/Users/johnlindquist/.codex/log/codex-tui.log:2075480:+        ["Contents/MacOS", ".app directory", "pasteboard fileURL"],
/Users/johnlindquist/.codex/log/codex-tui.log:2075487:+        ["drag payload must be the host .app directory, never the internal Contents/MacOS executable"]
/Users/johnlindquist/.codex/log/codex-tui.log:2076037:2026-05-15T18:52:13.585274Z  INFO codex_rmcp_client::logging_client_handler: MCP server log message (level: Info, logger: None, data: {"text":"Fea
... truncated ...
/Users/johnlindquist/.codex/log/codex-tui.log:3463184:2026-05-22T17:50:37.099166Z  INFO session_loop{thread_id=019e50ce-54c5-7422-b852-060ca93a114b}:submission_dispatch{otel.name="op.dispatch.user_inp
... truncated ...
... truncated, 7 more lines omitted
```

#### Tool 5 — Shell

```sh
find /Users/johnlindquist/.codex/cache -type f -newermt '2026-05-22 11:45:00' ! -newermt '2026-05-22 11:52:30' -print -exec stat -f '  birth:%SB modify:%Sm size:%z' {} \;
```

Result: completed, 4 lines

```text
/Users/johnlindquist/.codex/cache/codex_apps_tools/10c4c633c21af3657c1918b34b140d201ea68ea9.json
birth:Mar 28 13:43:08 2026 modify:May 22 11:51:41 2026 size:344723
/Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json
birth:May 14 15:00:50 2026 modify:May 22 11:49:33 2026 size:1287057
```

#### Tool 6 — Shell

```sh
find /Users/johnlindquist/.codex/sessions/2026/05/22 -type f -newermt '2026-05-22 11:45:00' ! -newermt '2026-05-22 11:52:30' -maxdepth 1 -print -exec stat -f '  birth:%SB modify:%Sm size:%z' {} \;
```

Result: completed, first 8 of 12 lines shown

```text
/Users/johnlindquist/.codex/sessions/2026/05/22/rollout-2026-05-22T11-48-47-019e50cd-d63b-7fb0-887d-19a74f0adfd7.jsonl
birth:May 22 11:48:47 2026 modify:May 22 11:48:50 2026 size:22173
/Users/johnlindquist/.codex/sessions/2026/05/22/rollout-2026-05-22T11-51-40-019e50d0-7b24-7473-85e3-7999446432a3.jsonl
birth:May 22 11:51:41 2026 modify:May 22 11:51:44 2026 size:22088
/Users/johnlindquist/.codex/sessions/2026/05/22/rollout-2026-05-22T08-11-11-019e5006-9eb5-75f0-b9e4-01835fa2f700.jsonl
birth:May 22 08:11:14 2026 modify:May 22 11:49:40 2026 size:6394000
/Users/johnlindquist/.codex/sessions/2026/05/22/rollout-2026-05-22T11-49-19-019e50ce-54c5-7422-b852-060ca93a114b.jsonl
birth:May 22 11:50:29 2026 modify:May 22 11:52:11 2026 size:207148
... truncated, 4 more lines omitted
```

#### Tool 7 — Shell

```sh
sed -n '3463000,3463188p' /Users/johnlindquist/.codex/log/codex-tui.log
```

Result: completed, first 8 of 129 lines shown

```text
Total output lines: 189

2026-05-22T17:49:20.070251Z  WARN codex_core_plugins::loader: configured non-curated plugin no longer exists in discovered marketplaces during cache refresh plugin="toolsmith" marketplace="better-plugins"
2026-05-22T17:49:20.080757Z  INFO thread_spawn{otel.name="thread_spawn"}:session_init:session_init.plugin_skill_warmup{otel.name="session_init.plugin_skill_warmup"}: codex_core::session::session: close time.busy=38.2ms time.idle=101ms
2026-05-22T17:49:20.080917Z  INFO codex_core_skills::manager: skills cache cleared (2 entries)
2026-05-22T17:49:20.081091Z  INFO thread_spawn{otel.name="thread_spawn"}:session_init:shell_snapshot{thread_id=019e50ce-54c5-7422-b852-060ca93a114b}: codex_core::shell_snapshot: new
2026-05-22T17:49:20.081100Z  INFO thread_spawn{otel.name="thread_spawn"}:session_init:session_init.thread_name_lookup{otel.name="session_init.thread_name_lookup"}: codex_core::session::session: new
2026-05-22T17:49:20.088234Z  INFO codex_core_plugins::remote::remote_installed_plugin_sync: completed remote installed plugin bundle sync installed_plugin_ids=[] removed_cache_plugin_ids=[] failed_remote_plugin_ids=[]
... truncated, 121 more lines omitted
```

#### Tool 8 — Shell

```sh
rg -n "2026-05-22T17:49|2026-05-22T17:50|app_directory|codex_app_directory|apps|connector|asdk_app|App" /Users/johnlindquist/.codex/log/codex-tui.log | tail -200
```

Result: completed, first 8 of 88 lines shown

```text
Total output lines: 200

3463072:2026-05-22T17:49:22.896204Z  INFO codex_core_plugins::remote::remote_installed_plugin_sync: completed remote installed plugin bundle sync installed_plugin_ids=[] removed_cache_plugin_ids=[] failed_remote_plugin_ids=[]
3463073:2026-05-22T17:49:23.118450Z  INFO model_client.stream_responses_websocket{model=gpt-5.5 wire_api=responses transport="responses_websocket" api.path="responses" turn.has_metadata_header=true we
... truncated ...
3463074:2026-05-22T17:49:23.480636Z  WARN codex_core_plugins::loader: skipping duplicate plugin MCP server name plugin="computer-use@openai-bundled-beta" previous_plugin="computer-use@openai-bundled" server="computer-use"
3463075:2026-05-22T17:49:23.845211Z  INFO model_client.stream_responses_websocket{model=gpt-5.5 wire_api=responses transport="responses_websocket" api.path="responses" turn.has_metadata_header=true we
... truncated ...
3463076:2026-05-22T17:49:25.011897Z  WARN codex_core_plugins::loader: skipping duplicate plugin MCP server name plugin="computer-use@openai-bundled-beta" previous_plugin="computer-use@openai-bundled" server="computer-use"
... truncated, 81 more lines omitted
```

#### Tool 9 — Shell

```sh
head -80 /Users/johnlindquist/.codex/sessions/2026/05/22/rollout-2026-05-22T11-48-47-019e50cd-d63b-7fb0-887d-19a74f0adfd7.jsonl
```

Result: completed, first 8 of 11 lines shown

```text
{"timestamp":"2026-05-22T17:48:47.746Z","type":"session_meta","payload":{"id":"019e50cd-d63b-7fb0-887d-19a74f0adfd7","timestamp":"2026-05-22T17:48:47.566Z","cwd":"/Users/johnlindquist/dev/script-kit-g
... truncated ...
{"timestamp":"2026-05-22T17:48:47.748Z","type":"event_msg","payload":{"type":"task_started","turn_id":"019e50cd-d6ef-7a71-8be2-b6b3963be3e4","started_at":1779472127,"model_context_window":121600,"collaboration_mode_kind":"default"}}
{"timestamp":"2026-05-22T17:48:49.591Z","type":"response_item","payload":{"type":"message","role":"developer","content":[{"type":"input_text","text":"<permissions instructions>\nFilesystem sandboxing
... truncated ...
{"timestamp":"2026-05-22T17:48:49.591Z","type":"turn_context","payload":{"turn_id":"019e50cd-d6ef-7a71-8be2-b6b3963be3e4","cwd":"/Users/johnlindquist/dev/script-kit-gpui","current_date":"2026-05-22","
... truncated ...
{"timestamp":"2026-05-22T17:48:49.593Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"Create a concise cmux tab title for this Codex conversa
... truncated ...
... truncated, 6 more lines omitted
```

#### Tool 10 — Shell

```sh
nl -ba /Users/johnlindquist/.codex/history.jsonl | sed -n '2320,2332p'
```

Result: completed, first 8 of 11 lines shown

```text
2320	{"session_id":"019e5006-9eb5-75f0-b9e4-01835fa2f700","ts":1779464718,"text":"On the footer buttons, without adjust the footer height, adjust the height of the buttons in the footer So they stay v
... truncated ...
2321	{"session_id":"019e5006-9eb5-75f0-b9e4-01835fa2f700","ts":1779465585,"text":"Treat the text inside of the buttons as if they have the same border/padding as the keyboard shortcuts so that the but
... truncated ...
2322	{"session_id":"019e5006-9eb5-75f0-b9e4-01835fa2f700","ts":1779467770,"text":"[Image #1] the buttons need more top/bottom padding to maintain a \"square\" shape and Mimic the look of a keyboard ke
... truncated ...
2323	{"session_id":"019e5006-9eb5-75f0-b9e4-01835fa2f700","ts":1779467891,"text":"[Image #1] Is there a background color applied to the footer? There shouldn't be.  Any color would block the beautiful vibrancy effect we've set up"}
2324	{"session_id":"019e5006-9eb5-75f0-b9e4-01835fa2f700","ts":1779467955,"text":"[Image #1] Same with the actions menu popup, does it have a background color? It shouldn't, the material should be doing all the background work we need"}
... truncated, 6 more lines omitted
```

#### Tool 11 — Shell

```sh
stat -f '%N%n  birth:  %SB%n  modify: %Sm%n  change: %Sc%n  access: %Sa%n  size:   %z' /Users/johnlindquist/.codex/cache/codex_apps_tools/10c4c633c21af3657c1918b34b140d201ea68ea9.json && rg -n "asdk_app_69a66c553c248191b2b2bc556373335b|Beast Academy|beast" /Users/johnlindquist/.codex/cache/codex_apps_tools/10c4c633c21af3657c1918b34b140d201ea68ea9.json
```

Result: completed, 6 lines

```text
/Users/johnlindquist/.codex/cache/codex_apps_tools/10c4c633c21af3657c1918b34b140d201ea68ea9.json
birth:  Mar 28 13:43:08 2026
modify: May 22 11:51:41 2026
change: May 22 11:51:41 2026
access: May 22 11:51:42 2026
size:   344723
```

#### Tool 12 — Shell

```sh
tail -120 /Users/johnlindquist/.codex/sessions/2026/05/22/rollout-2026-05-22T08-11-11-019e5006-9eb5-75f0-b9e4-01835fa2f700.jsonl
```

Result: completed, first 8 of 46 lines shown

```text
Total output lines: 120

{"timestamp":"2026-05-22T16:50:46.574Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":39003294,"cached_input_tokens":37491968,"output_tokens":75789,"re
... truncated ...
{"timestamp":"2026-05-22T16:50:49.696Z","type":"response_item","payload":{"type":"reasoning","summary":[],"content":null,"encrypted_content":"gAAAAABqEIlpDwwBZ986fU1Un05-zpesQ_xnYhAIn_9I4lHIMC9yEwpbNh
... truncated ...
{"timestamp":"2026-05-22T16:50:54.974Z","type":"event_msg","payload":{"type":"agent_message","message":"The main menu footer is an AppKit `NSVisualEffectView`, not a GPUI footer band. The likely culpr
... truncated ...
{"timestamp":"2026-05-22T16:50:54.975Z","type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"The main menu footer is an AppKit `NSVisualEffect
... truncated ...
{"timestamp":"2026-05-22T16:50:54.990Z","type":"response_item","payload":{"type":"function_call","name":"exec_command","arguments":"{\"cmd\":\"sed -n '380,640p' src/footer_popup.rs\",\"workdir\":\"/Us
... truncated ...
... truncated, 39 more lines omitted
```

#### Tool 13 — Shell

```sh
jq -r '.[]? // empty | select(.id=="asdk_app_69a66c553c248191b2b2bc556373335b")' /Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json
```

Result: completed, 1 line

```text
jq: error (at /Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json:40412): Cannot index number with string "id"
```

#### Tool 14 — Shell

```sh
find /Users/johnlindquist/.codex/cache/codex_app_directory -type f -exec sh -c 'for f; do printf "%s\n" "$f"; stat -f "  birth:%SB modify:%Sm size:%z" "$f"; done' sh {} +
```

Result: completed, 2 lines

```text
/Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json
birth:May 14 15:00:50 2026 modify:May 22 11:49:33 2026 size:1287057
```

#### Tool 15 — Shell

```sh
rg -n "asdk_app_69a66c553c248191b2b2bc556373335b|Beast Academy|beast-academy" /Users/johnlindquist/.codex/sessions/2026/05/22 /Users/johnlindquist/.codex/history.jsonl /Users/johnlindquist/.codex/cache 2>/dev/null
```

Result: completed, first 8 of 18 lines shown

```text
Total output lines: 32

/Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json:5179:      "id": "asdk_app_69a66c553c248191b2b2bc556373335b",
/Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json:5180:      "name": "Beast Academy - All Ten",
/Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json:5182:      "logoUrl": "connectors://asdk_app_69a66c553c248191b2b2bc556373335b/logo?theme=light",
/Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json:5207:      "installUrl": "https://chatgpt.com/apps/beast-academy---all-ten/asdk_app_69a66c553c248191b2b2bc556373335b",
/Users/johnlindquist/.codex/history.jsonl:2329:{"session_id":"019e50ce-54c5-7422-b852-060ca93a114b","ts":1779472229,"text":"[$beast-academy---all-ten](app://asdk_app_69a66c553c248191b2b2bc556373335b) <--- where is this skill coming from?"}
... truncated, 11 more lines omitted
```

#### Tool 16 — Shell

```sh
jq -r 'type, length, (.[0] | keys_unsorted? // empty)' /Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json
```

Result: completed, 3 lines

```text
jq: error (at /Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json:40412): Cannot index object with number
object
2
```

#### Tool 17 — Shell

```sh
head -20 /Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json
```

Result: completed, first 8 of 20 lines shown

```text
{
"schema_version": 1,
"connectors": [
{
"id": "asdk_app_698cee7c18848191b0568392d52f01ed",
"name": "% Arabica UAE",
"description": "Order % Arabica coffee UAE",
"logoUrl": "connectors://asdk_app_698cee7c18848191b0568392d52f01ed/logo?theme=light",
... truncated, 12 more lines omitted
```

#### Tool 18 — Shell

```sh
jq -r 'keys_unsorted[]' /Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json
```

Result: completed, 2 lines

```text
schema_version
connectors
```

#### Tool 19 — Shell

```sh
jq -r '.connectors | length' /Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json
```

Result: completed, 1 line

```text
1186
```

#### Tool 20 — Shell

```sh
jq '.connectors[] | select(.id=="asdk_app_69a66c553c248191b2b2bc556373335b")' /Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json
```

Result: completed, first 8 of 34 lines shown

```text
{
"id": "asdk_app_69a66c553c248191b2b2bc556373335b",
"name": "Beast Academy - All Ten",
"description": "Play today's number puzzle",
"logoUrl": "connectors://asdk_app_69a66c553c248191b2b2bc556373335b/logo?theme=light",
"logoUrlDark": null,
"distributionChannel": "ECOSYSTEM_DIRECTORY",
"branding": null,
... truncated, 26 more lines omitted
```

#### Tool 21 — Shell

```sh
jq -r '.connectors[] | select(.isAccessible==true) | [.name,.id] | @tsv' /Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json | head -20
```

Result: completed, no output.

#### Tool 22 — Shell

```sh
find /Users/johnlindquist/.codex/cache -maxdepth 2 -type f -name '*.json' -print -exec stat -f '  birth:%SB modify:%Sm size:%z' {} \; | sed -n '1,120p'
```

Result: completed, 4 lines

```text
/Users/johnlindquist/.codex/cache/codex_apps_tools/10c4c633c21af3657c1918b34b140d201ea68ea9.json
birth:Mar 28 13:43:08 2026 modify:May 22 11:51:41 2026 size:344723
/Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json
birth:May 14 15:00:50 2026 modify:May 22 11:49:33 2026 size:1287057
```

#### Tool 23 — Shell

```sh
jq -r '[.connectors[] | select(.isAccessible==true)] | length, [.connectors[] | select(.isEnabled==true)] | length, [.connectors[] | select(.distributionChannel=="ECOSYSTEM_DIRECTORY")] | length' /Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json
```

Result: completed, 2 lines

```text
jq: error (at /Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json:40412): Cannot index number with string "connectors"
0
```

#### Tool 24 — Shell

```sh
rg -n "codex_app_directory|codex_apps_tools|ECOSYSTEM_DIRECTORY|remote_installed_plugin_sync|connectors" /Users/johnlindquist/dev/codex /Users/johnlindquist/.codex /Users/johnlindquist/dev/script-kit-gpui/.codex 2>/dev/null | head -200
```

Result: completed, no output.

#### Tool 25 — Shell

```sh
nl -ba /Users/johnlindquist/.codex/memories/MEMORY.md | sed -n '2228,2278p'
```

Result: completed, first 8 of 51 lines shown

```text
2228
2229	### keywords
2230
2231	- sessions, log/codex-tui.log, generated_images, shell_snapshots, state_5.sqlite, logs_2.sqlite, worktrees, cache/codex_apps_tools, expected live state
2232
2233	## Task 3: Classify `~/.codex` paths against `~/dev/codex` source, success
2234
2235	### rollout_summary_files
... truncated, 43 more lines omitted
```

#### Tool 26 — Shell

```sh
jq -r '[(.connectors[] | select(.isAccessible==true))] | length' /Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json && jq -r '[(.connectors[] | select(.isEnabled==true))] | length' /Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json && jq -r '[(.connectors[] | select(.distributionChannel=="ECOSYSTEM_DIRECTORY"))] | length' /Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json
```

Result: completed, 3 lines

```text
0
1186
1159
```

### Assistant

It looks like this was not manually installed.

The short version:

- `Beast Academy - All Ten` came from Codex’s cached **ecosystem app directory**, not from repo skills or installed plugins.
- The cache file is [b7d8cf...json](/Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json:5179).
- That file was created `May 14 15:00:50 2026 MDT` and rewritten today at `May 22 11:49:33 2026 MDT`.
- Your prompt containing it was recorded at `May 22 11:50:29 2026 MDT` in [history.jsonl](/Users/johnlindquist/.codex/history.jsonl:2329).
- The entry says `distributionChannel: "ECOSYSTEM_DIRECTORY"`, `isAccessible: false`, `isEnabled: true`.
- The full directory cache has `1186` connectors, `0` accessible connectors, and `1159` ecosystem-directory entries.

Timeline I found:

- `11:49:20 MDT`: new Codex session init begins; logs show plugin/cache refresh activity and `remote_installed_plugin_sync` with `installed_plugin_ids=[]`.
- `11:49:33 MDT`: `~/.codex/cache/codex_app_directory/...json` modified.
- `11:50:29 MDT`: your prompt is stored as `[$beast-academy---all-ten](app://asdk_app_69a66c553c248191b2b2bc556373335b)`.
- `11:51:41 MDT`: `cache/codex_apps_tools/...json` modified, but that file does not contain Beast Academy.

My read: Codex refreshed its ChatGPT Apps / connector directory cache during session startup, and the composer resolved `$beast-academy---all-ten` against that directory. It did not come from `.agents/skills`, `.codex/skills`, or an installed plugin. I can’t prove whether Beast Academy was already present in the May 14 version of the cache without an older snapshot/back-up, because the cache file was overwritten today.

## Round 3

### You

[Image #1] Like, where did these ALL come from? It feels like I either installed something malicious or codex is now spamming apps

### Tools

#### Tool 1 — Shell

```sh
jq -r '.connectors[] | select((.name|ascii_downcase|startswith("be")) or (.installUrl|contains("/be"))) | [.name,.description,.isAccessible,.isEnabled,.distributionChannel,.id] | @tsv' /Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json | head -80
```

Result: completed, 8 lines

```text
BE-A	Real-time Direct Hotel Booking	false	true	ECOSYSTEM_DIRECTORY	asdk_app_69a0553490f08191b4959d30ed802a1e
Beast Academy - All Ten	Play today's number puzzle	false	true	ECOSYSTEM_DIRECTORY	asdk_app_69a66c553c248191b2b2bc556373335b
Beautiful.ai	Turn ideas into presentations	false	true	ECOSYSTEM_DIRECTORY	asdk_app_69433d565fa8819193c333c261b7d8a2
Bed-and-Breakfast.it	Book Italian stays direct	false	true	ECOSYSTEM_DIRECTORY	asdk_app_695e7c4731d4819185471e24b5d8a3f9
Bee Spelling	Test your spelling	false	true	ECOSYSTEM_DIRECTORY	asdk_app_69d642b6515c81918a1d61bf2467a088
Best Lawyers	Search peer-recognized lawyers	false	true	ECOSYSTEM_DIRECTORY	asdk_app_699893175c08819192759ab4dc160abf
BestColleges.com	Find schools and programs	false	true	ECOSYSTEM_DIRECTORY	asdk_app_6970e5fa955c819195ad047fb9430ddd
BestMoney - Car Insurance	Your shortcut to car quotes	false	true	ECOSYSTEM_DIRECTORY	asdk_app_69ee0db9dbac8191ac4f6171a2f4c074
```

#### Tool 2 — Shell

```sh
stat -f '%N%n  birth: %SB%n  modify:%Sm%n  size:  %z' /Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json /Users/johnlindquist/.codex/config.toml
```

Result: completed, 8 lines

```text
/Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json
birth: May 14 15:00:50 2026
modify:May 22 11:49:33 2026
size:  1287057
/Users/johnlindquist/.codex/config.toml
birth: May 21 21:31:28 2026
modify:May 21 21:31:28 2026
size:  14599
```

#### Tool 3 — Shell

```sh
jq -r '.connectors[] | select(.name=="BE-A" or .name=="Beast Academy - All Ten" or .name=="Beautiful.ai" or .name=="Bed-and-Breakfast.it" or .name=="Bee Spelling" or .name=="BestMoney - Car Insurance") | [.name,.description,.isAccessible,.isEnabled,.distributionChannel,.installUrl] | @tsv' /Users/johnlindquist/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json
```

Result: completed, 6 lines

```text
BE-A	Real-time Direct Hotel Booking	false	true	ECOSYSTEM_DIRECTORY	https://chatgpt.com/apps/be-a/asdk_app_69a0553490f08191b4959d30ed802a1e
Beast Academy - All Ten	Play today's number puzzle	false	true	ECOSYSTEM_DIRECTORY	https://chatgpt.com/apps/beast-academy---all-ten/asdk_app_69a66c553c248191b2b2bc556373335b
Beautiful.ai	Turn ideas into presentations	false	true	ECOSYSTEM_DIRECTORY	https://chatgpt.com/apps/beautiful-ai/asdk_app_69433d565fa8819193c333c261b7d8a2
Bed-and-Breakfast.it	Book Italian stays direct	false	true	ECOSYSTEM_DIRECTORY	https://chatgpt.com/apps/bed-and-breakfast-it/asdk_app_695e7c4731d4819185471e24b5d8a3f9
Bee Spelling	Test your spelling	false	true	ECOSYSTEM_DIRECTORY	https://chatgpt.com/apps/bee-spelling/asdk_app_69d642b6515c81918a1d61bf2467a088
BestMoney - Car Insurance	Your shortcut to car quotes	false	true	ECOSYSTEM_DIRECTORY	https://chatgpt.com/apps/bestmoney---car-insurance/asdk_app_69ee0db9dbac8191ac4f6171a2f4c074
```

#### Tool 4 — Shell

```sh
sed -n '1,110p' /Users/johnlindquist/dev/codex/codex-rs/connectors/src/lib.rs
```

Result: completed, first 8 of 110 lines shown

```text
use std::collections::HashMap;
use std::future::Future;
use std::sync::LazyLock;
use std::sync::Mutex as StdMutex;
use std::time::Duration;
use std::time::Instant;

use codex_app_server_protocol::AppBranding;
... truncated, 102 more lines omitted
```

#### Tool 5 — Shell

```sh
sed -n '1,130p' /Users/johnlindquist/dev/codex/codex-rs/connectors/src/merge.rs
```

Result: completed, first 8 of 130 lines shown

```text
use std::collections::HashMap;
use std::collections::HashSet;

use crate::metadata::connector_install_url;
use crate::metadata::sort_connectors_by_accessibility_and_name;
use codex_app_server_protocol::AppInfo;

pub fn merge_connectors(
... truncated, 122 more lines omitted
```

#### Tool 6 — Shell

```sh
sed -n '1,240p' /Users/johnlindquist/dev/codex/codex-rs/connectors/src/filter.rs
```

Result: completed, first 8 of 227 lines shown

```text
use std::collections::HashSet;

use codex_app_server_protocol::AppInfo;

pub fn filter_tool_suggest_discoverable_connectors(
directory_connectors: Vec<AppInfo>,
accessible_connectors: &[AppInfo],
discoverable_connector_ids: &HashSet<String>,
... truncated, 219 more lines omitted
```

#### Tool 7 — Shell

```sh
sed -n '1,140p' /Users/johnlindquist/dev/codex/codex-rs/connectors/src/directory_cache.rs
```

Result: completed, first 8 of 112 lines shown

```text
use std::path::PathBuf;

use codex_app_server_protocol::AppInfo;
use serde::Deserialize;
use serde::Serialize;
use sha1::Digest;
use sha1::Sha1;
use tracing::warn;
... truncated, 104 more lines omitted
```

#### Tool 8 — Shell

```sh
rg -n "filter_tool_suggest_discoverable_connectors|load_cached_directory_connectors_from_disk|write_cached_directory_connectors_to_disk|merge_connectors\(" /Users/johnlindquist/dev/codex/codex-rs -g '*.rs'
```

Result: completed, first 8 of 16 lines shown

```text
/Users/johnlindquist/dev/codex/codex-rs/connectors/src/directory_cache.rs:44:pub(crate) fn load_cached_directory_connectors_from_disk(
/Users/johnlindquist/dev/codex/codex-rs/connectors/src/directory_cache.rs:82:pub(crate) fn write_cached_directory_connectors_to_disk(
/Users/johnlindquist/dev/codex/codex-rs/connectors/src/filter.rs:5:pub fn filter_tool_suggest_discoverable_connectors(
/Users/johnlindquist/dev/codex/codex-rs/connectors/src/filter.rs:161:    fn filter_tool_suggest_discoverable_connectors_keeps_only_plugin_backed_uninstalled_apps() {
/Users/johnlindquist/dev/codex/codex-rs/connectors/src/filter.rs:162:        let filtered = filter_tool_suggest_discoverable_connectors(
/Users/johnlindquist/dev/codex/codex-rs/connectors/src/filter.rs:195:    fn filter_tool_suggest_discoverable_connectors_excludes_accessible_apps_even_when_disabled() {
/Users/johnlindquist/dev/codex/codex-rs/connectors/src/filter.rs:196:        let filtered = filter_tool_suggest_discoverable_connectors(
/Users/johnlindquist/dev/codex/codex-rs/connectors/src/merge.rs:8:pub fn merge_connectors(
... truncated, 8 more lines omitted
```

#### Tool 9 — Shell

```sh
sed -n '110,200p' /Users/johnlindquist/dev/codex/codex-rs/core/src/connectors.rs
```

Result: completed, first 8 of 91 lines shown

```text
}

pub(crate) async fn list_tool_suggest_discoverable_tools_with_auth(
config: &Config,
auth: Option<&CodexAuth>,
accessible_connectors: &[AppInfo],
) -> anyhow::Result<Vec<DiscoverableTool>> {
let connector_ids = tool_suggest_connector_ids(config).await;
... truncated, 83 more lines omitted
```

#### Tool 10 — Shell

```sh
sed -n '150,205p' /Users/johnlindquist/dev/codex/codex-rs/connectors/src/lib.rs
```

Result: completed, first 8 of 55 lines shown

```text
let mut connectors = merge_directory_apps(apps)
.into_iter()
.map(directory_app_to_app_info)
.collect::<Vec<_>>();
for connector in &mut connectors {
let install_url = match connector.install_url.take() {
Some(install_url) => install_url,
None => connector_install_url(&connector.name, &connector.id),
... truncated, 47 more lines omitted
```

#### Tool 11 — Shell

```sh
sed -n '170,210p' /Users/johnlindquist/dev/codex/codex-rs/chatgpt/src/connectors.rs
```

Result: completed, first 8 of 41 lines shown

```text
.filter(|connector| plugin_app_ids.contains(connector.id.as_str()))
.collect()
}

pub fn merge_connectors_with_accessible(
connectors: Vec<AppInfo>,
accessible_connectors: Vec<AppInfo>,
all_connectors_loaded: bool,
... truncated, 33 more lines omitted
```

#### Tool 12 — Shell

```sh
rg -n "ToolSuggest|tool_suggest|discoverable_connector|codex_app_directory|connectors" /Users/johnlindquist/dev/codex/codex-rs -g '*.rs' | head -200
```

Result: completed, first 8 of 200 lines shown

```text
/Users/johnlindquist/dev/codex/codex-rs/connectors/src/directory_cache.rs:13:const CONNECTOR_DIRECTORY_DISK_CACHE_DIR: &str = "cache/codex_app_directory";
/Users/johnlindquist/dev/codex/codex-rs/connectors/src/directory_cache.rs:39:    Hit { connectors: Vec<AppInfo> },
/Users/johnlindquist/dev/codex/codex-rs/connectors/src/directory_cache.rs:44:pub(crate) fn load_cached_directory_connectors_from_disk(
/Users/johnlindquist/dev/codex/codex-rs/connectors/src/directory_cache.rs:78:        connectors: cache.connectors,
/Users/johnlindquist/dev/codex/codex-rs/connectors/src/directory_cache.rs:82:pub(crate) fn write_cached_directory_connectors_to_disk(
/Users/johnlindquist/dev/codex/codex-rs/connectors/src/directory_cache.rs:84:    connectors: &[AppInfo],
/Users/johnlindquist/dev/codex/codex-rs/connectors/src/directory_cache.rs:94:        connectors: connectors.to_vec(),
/Users/johnlindquist/dev/codex/codex-rs/connectors/src/directory_cache.rs:104:    connectors: Vec<AppInfo>,
... truncated, 192 more lines omitted
```

#### Tool 13 — Shell

```sh
sed -n '1190,1235p' /Users/johnlindquist/dev/codex/codex-rs/core/src/session/turn.rs
```

Result: completed, first 8 of 46 lines shown

```text
.services
.plugins_manager
.plugins_for_config(&turn_context.config.plugins_config_input())
.await;

let apps_enabled = turn_context.apps_enabled();
let accessible_connectors =
apps_enabled.then(|| connectors::accessible_connectors_from_mcp_tools(&all_mcp_tools));
... truncated, 38 more lines omitted
```

#### Tool 14 — Shell

```sh
sed -n '1410,1465p' /Users/johnlindquist/dev/codex/codex-rs/core/src/session/tests.rs && sed -n '1988,2010p' /Users/johnlindquist/dev/codex/codex-rs/core/src/session/tests.rs
```

Result: completed, first 8 of 78 lines shown

```text
assert_eq!(session.hooks().preview_session_start(&request).len(), 1);
Ok(())
}

#[tokio::test]
async fn reload_user_config_layer_updates_effective_tool_suggest_config() {
let (session, _turn_context) = make_session_and_context().await;
let codex_home = session.codex_home().await;
... truncated, 70 more lines omitted
```

#### Tool 15 — Shell

```sh
rg -n "struct ToolSuggest|tool_suggest|disabled_tools|ToolSuggestDisabledTool|tool_suggest_enabled" /Users/johnlindquist/dev/codex/codex-rs/config /Users/johnlindquist/dev/codex/codex-rs/core/src /Users/johnlindquist/dev/codex/codex-rs/tools -g '*.rs'
```

Result: completed, first 8 of 248 lines shown

```text
/Users/johnlindquist/dev/codex/codex-rs/config/src/mcp_types.rs:173:    pub disabled_tools: Option<Vec<String>>,
/Users/johnlindquist/dev/codex/codex-rs/config/src/mcp_types.rs:253:    pub disabled_tools: Option<Vec<String>>,
/Users/johnlindquist/dev/codex/codex-rs/config/src/mcp_types.rs:291:            disabled_tools,
/Users/johnlindquist/dev/codex/codex-rs/config/src/mcp_types.rs:364:            disabled_tools,
/Users/johnlindquist/dev/codex/codex-rs/core/src/session/tests.rs:22:use codex_config::types::ToolSuggestDisabledTool;
/Users/johnlindquist/dev/codex/codex-rs/core/src/session/tests.rs:1416:async fn reload_user_config_layer_updates_effective_tool_suggest_config() {
/Users/johnlindquist/dev/codex/codex-rs/core/src/session/tests.rs:1423:        r#"[tool_suggest]
/Users/johnlindquist/dev/codex/codex-rs/core/src/session/tests.rs:1424:disabled_tools = [
... truncated, 240 more lines omitted
```

#### Tool 16 — Shell

```sh
rg -n "\[tool_suggest\]|disabled_tools|apps|connectors|features" /Users/johnlindquist/.codex/config.toml
```

Result: completed, 2 lines

```text
4:suppress_unstable_features_warning = true
10:[features]
```

#### Tool 17 — Shell

```sh
sed -n '200,270p' /Users/johnlindquist/dev/codex/codex-rs/core/src/tools/spec_plan.rs
```

Result: completed, first 8 of 71 lines shown

```text
}

fn spec_for_model_request(
turn_context: &TurnContext,
exposure: ToolExposure,
spec: ToolSpec,
) -> ToolSpec {
if code_mode_enabled(turn_context)
... truncated, 63 more lines omitted
```

#### Tool 18 — Shell

```sh
sed -n '400,455p' /Users/johnlindquist/dev/codex/codex-rs/core/src/connectors.rs
```

Result: completed, first 8 of 56 lines shown

```text
*cache_guard = Some(CachedAccessibleConnectors {
key: cache_key,
expires_at: Instant::now() + codex_connectors::CONNECTORS_CACHE_TTL,
connectors: connectors.to_vec(),
});
}

async fn tool_suggest_connector_ids(config: &Config) -> HashSet<String> {
... truncated, 48 more lines omitted
```

#### Tool 19 — Shell

```sh
sed -n '1170,1230p' /Users/johnlindquist/dev/codex/codex-rs/core/src/connectors_tests.rs
```

Result: completed, first 8 of 61 lines shown

```text
AppToolPolicy {
enabled: true,
approval: AppToolApproval::Approve,
}
);
}

#[tokio::test]
... truncated, 53 more lines omitted
```

#### Tool 20 — Shell

```sh
sed -n '200,265p' /Users/johnlindquist/dev/codex/codex-rs/config/src/types.rs
```

Result: completed, first 8 of 66 lines shown

```text
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ToolSuggestDiscoverableType {
Connector,
Plugin,
}
... truncated, 58 more lines omitted
```

#### Tool 21 — Shell

```sh
sed -n '1,45p' /Users/johnlindquist/.codex/config.toml
```

Result: completed, first 8 of 45 lines shown

```text
model = "gpt-5.5"
model_reasoning_effort = "medium"
personality = "pragmatic"
suppress_unstable_features_warning = true
allow_login_shell = true

notify = ["/Users/johnlindquist/.codex/computer-use/Codex Computer Use.app/Contents/SharedSupport/SkyComputerUseClient.app/Contents/MacOS/SkyComputerUseClient", "turn-ended"]
approvals_reviewer = "user"
... truncated, 37 more lines omitted
```

#### Tool 22 — Shell

```sh
rg -n "discoverables|disabled_tools|tool_suggest" /Users/johnlindquist/.codex/config.toml /Users/johnlindquist/dev/script-kit-gpui/.codex/config.toml /Users/johnlindquist/dev/codex/.codex/config.toml 2>/dev/null
```

Result: completed, no output.

#### Tool 23 — Shell

```sh
find /Users/johnlindquist/.codex/plugins/cache -name 'plugin.json' -print | head -200 | xargs rg -n 'app|connector|asdk_app'
```

Result: completed, first 8 of 25 lines shown

```text
/Users/johnlindquist/.codex/plugins/cache/openai-curated/slack/004da724/.codex-plugin/plugin.json:19:  "apps": "./.app.json",
/Users/johnlindquist/.codex/plugins/cache/openai-curated/slack/004da724/.codex-plugin/plugin.json:37:    "composerIcon": "./assets/app-icon.png",
/Users/johnlindquist/.codex/plugins/cache/openai-curated/slack/004da724/.codex-plugin/plugin.json:38:    "logo": "./assets/app-icon.png",
/Users/johnlindquist/.codex/plugins/cache/openai-curated/github/004da724/.codex-plugin/plugin.json:4:  "description": "Inspect repositories, triage pull requests and issues, debug CI, and publish chan
... truncated ...
/Users/johnlindquist/.codex/plugins/cache/openai-curated/github/004da724/.codex-plugin/plugin.json:22:  "apps": "./.app.json",
/Users/johnlindquist/.codex/plugins/cache/openai-curated/github/004da724/.codex-plugin/plugin.json:26:    "longDescription": "Use GitHub to inspect repositories, review pull requests, address feedback
... truncated ...
/Users/johnlindquist/.codex/plugins/cache/openai-bundled/computer-use/1.0.793/.codex-plugin/plugin.json:4:  "description": "Control desktop apps on macOS from Codex through Computer Use.",
... truncated, 18 more lines omitted
```

#### Tool 24 — Shell

```sh
rg -n "asdk_app_69a0553490f08191b4959d30ed802a1e|asdk_app_69a66c553c248191b2b2bc556373335b|asdk_app_69433d565fa8819193c333c261b7d8a2|asdk_app_695e7c4731d4819185471e24b5d8a3f9|asdk_app_69d642b6515c81918a1d61bf2467a088|asdk_app_699893175c08819192759ab4dc160abf" /Users/johnlindquist/.codex/plugins /Users/johnlindquist/.codex/.tmp /Users/johnlindquist/.codex/cache /Users/johnlindquist/dev/codex 2>/dev/null | head -100
```

Result: completed, first 8 of 33 lines shown

```text
/Users/johnlindquist/.codex/plugins/data/toolsmith-better-plugins/indexes/live-index.json:329:    "rg -n \"<str>\" /<path> /<path> /<path> /<path>": "rg -n \"asdk_app_69a66c553c248191b2b2bc556373335b|
... truncated ...
/Users/johnlindquist/.codex/plugins/data/toolsmith-better-plugins/indexes/live-index.json:332:    "rg -n \"<str>\" /<path> /<path> /<path>": "rg -n \"app directory|codex_app_directory|asdk_app_69a66c5
... truncated ...
/Users/johnlindquist/.codex/plugins/data/toolsmith-better-plugins/indexes/live-index.json:339:    "jq -r '<str>' /<path>": "jq -r '.[]? // empty | select(.id==\"asdk_app_69a66c553c248191b2b2bc55637333
... truncated ...
/Users/johnlindquist/.codex/plugins/data/toolsmith-better-plugins/indexes/live-index.json:340:    "rg -n \"<str>\" /<path> /<path> /<path> <num>>/dev/null": "rg -n \"asdk_app_69a66c553c248191b2b2bc556
... truncated ...
/Users/johnlindquist/.codex/plugins/data/toolsmith-better-plugins/indexes/live-index.json:341:    "jq '<str>' /<path>": "jq '.connectors[] | select(.id==\"asdk_app_69a66c553c248191b2b2bc556373335b\")'
... truncated ...
... truncated, 28 more lines omitted
```

#### Tool 25 — Shell

```sh
rg -n "DiscoverableTool|discoverable_tools|tool_suggest|ToolSuggest|apps_enabled|available_connectors" /Users/johnlindquist/dev/codex/codex-rs/tui /Users/johnlindquist/dev/codex/codex-rs/core/src -g '*.rs' | head -250
```

Result: completed, first 8 of 250 lines shown

```text
/Users/johnlindquist/dev/codex/codex-rs/core/src/session/mcp.rs:288:        let host_owned_codex_apps_enabled =
/Users/johnlindquist/dev/codex/codex-rs/core/src/session/mcp.rs:289:            host_owned_codex_apps_enabled(&mcp_config, auth.as_ref());
/Users/johnlindquist/dev/codex/codex-rs/core/src/session/mcp.rs:322:            host_owned_codex_apps_enabled,
/Users/johnlindquist/dev/codex/codex-rs/core/src/session/tests.rs:22:use codex_config::types::ToolSuggestDisabledTool;
/Users/johnlindquist/dev/codex/codex-rs/core/src/session/tests.rs:544:            discoverable_tools: None,
/Users/johnlindquist/dev/codex/codex-rs/core/src/session/tests.rs:1416:async fn reload_user_config_layer_updates_effective_tool_suggest_config() {
/Users/johnlindquist/dev/codex/codex-rs/core/src/session/tests.rs:1423:        r#"[tool_suggest]
/Users/johnlindquist/dev/codex/codex-rs/core/src/session/tests.rs:1436:        config.tool_suggest.disabled_tools,
... truncated, 242 more lines omitted
```

#### Tool 26 — Shell

```sh
rg -n "\[App\]|App\]|connector_mention_slug|app://|Mention|mention|Composer|autocomplete|completion|Completion" /Users/johnlindquist/dev/codex/codex-rs/tui /Users/johnlindquist/dev/codex/codex-rs -g '*.rs' | head -300
```

Result: completed, first 8 of 300 lines shown

```text
/Users/johnlindquist/dev/codex/codex-rs/analytics/src/facts.rs:331:    AppMentioned(AppMentionedInput),
/Users/johnlindquist/dev/codex/codex-rs/analytics/src/facts.rs:343:pub(crate) struct AppMentionedInput {
/Users/johnlindquist/dev/codex/codex-rs/analytics/src/facts.rs:345:    pub mentions: Vec<AppInvocation>,
/Users/johnlindquist/dev/codex/codex-rs/app-server-client/src/lib.rs:154:    // the TUI with permanently corrupted markdown, while dropping completion
/Users/johnlindquist/dev/codex/codex-rs/app-server-client/src/lib.rs:169:/// output or leaves surfaces waiting for a completion signal that already
/Users/johnlindquist/dev/codex/codex-rs/app-server-client/src/lib.rs:199:/// Lossless events (transcript deltas, item/turn completions) block until the
/Users/johnlindquist/dev/codex/codex-rs/app-server-client/src/lib.rs:218:            // us to drop the transcript/completion notification the caller is blocked on.
/Users/johnlindquist/dev/codex/codex-rs/app-server-client/src/lib.rs:252:        // Block until the consumer catches up for transcript/completion notifications; this
... truncated, 292 more lines omitted
```

### Assistant

This is not a skill and I don’t see evidence that you installed something malicious.

Those rows are coming from Codex’s ChatGPT Apps / connector directory cache:

`~/.codex/cache/codex_app_directory/b7d8cf0866ab0da129592cf7a860d948c75ddd88.json`

That cache currently contains `1186` directory apps. The screenshot rows all match entries in that file, including:

- `Beast Academy - All Ten`
- `Beautiful.ai`
- `Bed-and-Breakfast.it`
- `Bee Spelling`
- `Best Lawyers`

They are marked as `distributionChannel="ECOSYSTEM_DIRECTORY"` and `isAccessible=false`. The confusing bit is that they also have `isEnabled=true`, but in this context that does not mean “installed/authorized”; it means the directory listing entry is enabled for discovery. The decisive field is `isAccessible=false`.

Timeline I found:

- `~/.codex/cache/codex_app_directory/...json`
  - created: `May 14 15:00:50 2026`
  - modified: `May 22 11:49:33 2026 MDT`
- Your `$beast-academy---all-ten` message hit Codex history at:
  - `May 22 11:50:29 2026 MDT`

So Codex refreshed that app directory cache about one minute before you asked. The TUI log around that time shows a new session startup and remote plugin sync with:

`installed_plugin_ids=[]`

I also checked for those app IDs under plugin/cache locations and the actual `codex_apps_tools` cache. They only showed up in the app directory cache, not as installed plugin/tool definitions.

The source path is:

- Codex fetches `/connectors/directory/list?external_logos=true`
- writes it under `~/.codex/cache/codex_app_directory`
- then the `$` mention picker is surfacing those discoverable app entries as `[App]`

So yes, this is effectively Codex spamming directory-discoverable apps in the mention picker. It is not coming from `script-kit-gpui`, and it is not a local skill.

Best next fixes are either patch local Codex to hide non-accessible ecosystem directory apps from `$` suggestions, or configure/disable tool suggestions if the current feature flag path supports it cleanly. Deleting the cache would only be temporary; it will come back on refresh.
