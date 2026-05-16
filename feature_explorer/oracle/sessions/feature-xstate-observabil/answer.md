Launching browser mode (gpt-5.5-pro) with ~99,488 tokens.
This run can take up to an hour (usually ~10 minutes).
[browser] [phase] chrome-launch — 1011ms pid=8074 port=9223 reused=false
[browser] [phase] cdp-connect — 142ms targetId=EBB4445D90A8AD502E75DD207A5A42A1
[browser] [build] oracle build=0.9.0+2026-05-14T2001 pid=67531 chrome_pid=8074 port=9223
[browser] [phase] cookie-sync — 32ms count=26
[browser] [nav] login check passed (status=200, domLoginCta=false)
[browser] [phase] login — 107ms
[browser] [model] opening model picker
[browser] [model] selection complete
[browser] [phase] model-select — 65ms model=Use latest model strategy=select
[browser] [phase] thinking-time — 359ms level=extended
[browser] [phase] submit-flow — started chars=2553 attachments=1
[browser] [attach] waitForAttachmentCompletion finished
[browser] [submit] promptLength=2553, method=button
[browser] [submit] verifyPromptCommitted succeeded
[browser] [phase] submit-flow — 7288ms baseline_turns=3
[browser] [lifecycle] waiting for assistant response — timeout=5400000ms baseline_turns=3
[browser] [poll] start — timeout=5400000ms minTurn=3
[browser] [poll] state change at 0s cycle=1 stop=true completion=false turns=4 lastAsst=true copyGlobal=true copyInTurn=false composerReady=false
[browser] [response] evaluation won race — aborting poller
[browser] [response] evaluation parsed — 189 chars, messageId=none
[browser] [poll] aborted after 28 cycles
[browser] [response] post-eval state — stop=true completion=false turns=4 copyGlobal=true copyInTurn=false composerReady=false
[browser] [response] entering second poller — candidate=189 chars, timeout=2700000ms
[browser] [poll] start — timeout=2700000ms minTurn=3
[browser] [poll] state change at 0s cycle=1 stop=true completion=false turns=4 lastAsst=true copyGlobal=true copyInTurn=false composerReady=false
[browser] [poll] heartbeat at 31s cycle=33 stop=true completion=false turns=4 lastAsst=true copyGlobal=true copyInTurn=false composerReady=false stableNoStop=0 flickers=0 remaining=2669s
[browser] [poll] heartbeat at 91s cycle=93 stop=true completion=false turns=4 lastAsst=true copyGlobal=true copyInTurn=false composerReady=false stableNoStop=0 flickers=0 remaining=2609s
[browser] [poll] heartbeat at 151s cycle=153 stop=true completion=false turns=4 lastAsst=true copyGlobal=true copyInTurn=false composerReady=false stableNoStop=0 flickers=0 remaining=2549s
[browser] [poll] heartbeat at 212s cycle=213 stop=true completion=false turns=5 lastAsst=true copyGlobal=true copyInTurn=false composerReady=false stableNoStop=0 flickers=0 remaining=2488s
[browser] [poll] heartbeat at 272s cycle=273 stop=true completion=false turns=5 lastAsst=true copyGlobal=true copyInTurn=false composerReady=false stableNoStop=0 flickers=0 remaining=2428s
[browser] [poll] state change at 332s cycle=333 stop=false completion=true turns=6 lastAsst=true copyGlobal=true copyInTurn=true composerReady=false
[browser] [poll] heartbeat at 332s cycle=333 stop=false completion=true turns=6 lastAsst=true copyGlobal=true copyInTurn=true composerReady=false stableNoStop=1 flickers=0 remaining=2368s
[browser] [poll] completion detected at 332s cycle=333 — reading snapshot
[browser] [poll] snapshot captured — 17648 chars
[browser] [response] second poller completed — 17648 chars
[browser] [lifecycle] assistant response captured after 363s — 17648 chars
[browser] [phase] cleanup — 89ms
[browser] [phase] total — 373953ms status=complete

6m13s · gpt-5.5-pro[browser] · ↑99.49k ↓4.41k ↻0 Δ103.9k
files=1
