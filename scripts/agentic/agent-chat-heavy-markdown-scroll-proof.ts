#!/usr/bin/env bun
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver";

type Args = {
  binary: string;
  messageCount: number;
  scrollCycles: number;
  receipt: string;
  proveThumb: boolean;
};

const DEFAULT_RECEIPT = ".test-output/agent-chat-heavy-markdown-scroll-proof.json";

function parseArgs(): Args {
  const args = Bun.argv.slice(2);
  const get = (name: string, fallback?: string): string => {
    const ix = args.indexOf(name);
    if (ix >= 0 && args[ix + 1]) return args[ix + 1];
    if (fallback !== undefined) return fallback;
    throw new Error(`Missing required arg ${name}`);
  };
  return {
    binary: get("--binary", process.env.SCRIPT_KIT_GPUI_BINARY ?? "target/debug/script-kit-gpui"),
    messageCount: Number(get("--message-count", "160")),
    scrollCycles: Number(get("--scroll-cycles", "80")),
    receipt: get("--receipt", DEFAULT_RECEIPT),
    proveThumb: args.includes("--prove-thumb"),
  };
}

function heavyAssistantMarkdown(): string {
  return `# Stress Markdown Response

> Outer quote for layout stress with **bold**, _italic_, \`inline code\`, and a nested quote.
>
> > Nested quote with a long sentence that should wrap cleanly across the Agent Chat transcript viewport while retaining readable spacing and avoiding scroll jank.

- [x] Completed item with inline \`Result<T, E>\`
- [ ] Pending item with a [documentation-style link](https://example.com/docs/perf)
- Mixed **strong** and _emphasis_ plus ~~strikethrough~~ in one line.

| Column | Description | Value |
| --- | --- | ---: |
| Render | Markdown-heavy transcript row with table layout | 42 |
| Scroll | Rapid wheel input while visible rows recycle | 9001 |
| Wrap | Very long table content that forces the renderer to measure text repeatedly without reflowing the entire transcript | 123 |

\`\`\`rust
pub fn expensive_markdown_case(input: &str) -> anyhow::Result<()> {
    let spans = input.lines().enumerate().map(|(index, line)| {
        format!("{index:04}: {line}")
    }).collect::<Vec<_>>();

    for span in spans.iter().filter(|line| line.contains("markdown")) {
        tracing::debug!(%span, "render stress row");
    }

    Ok(())
}
\`\`\`

\`\`\`diff
- old transcript renderer eagerly rebuilt every row on scroll
+ current transcript renderer should keep markdown views stable
+ scroll tracing should show render timings during manual testing
\`\`\`

\`\`\`json
{
  "fixture": "agent-chat-heavy-markdown",
  "features": ["tables", "quotes", "code_fences", "lists", "links"],
  "scrollPerf": { "target": "manual", "trace": true }
}
\`\`\`

1. Ordered item with a paragraph after it.
2. Second ordered item with nested bullets:
   - nested bullet A with a deliberately lengthy phrase to exercise wrapping and markdown span measurement
   - nested bullet B with \`inline_symbol_names_that_are_long_enough_to_wrap\`
3. Final ordered item.

---

Final paragraph: this row intentionally combines many markdown constructs and enough text volume to stress layout, virtualization, render reuse, and scroll invalidation without depending on external network content.`;
}

function userMarkdown(): string {
  return "Please review this markdown-heavy response with code fences, tables, quotes, lists, links, and long wrapping text so we can stress Agent Chat scrolling.";
}

function percentile(values: number[], p: number): number {
  if (values.length === 0) return 0;
  const sorted = [...values].sort((a, b) => a - b);
  const ix = Math.min(sorted.length - 1, Math.round((sorted.length - 1) * p));
  return sorted[ix];
}

function parseRenderTrace(logPath: string): Json {
  const text = existsSync(logPath) ? readFileSync(logPath, "utf8") : "";
  const elapsed: number[] = [];
  const heavyPreviews: number[] = [];
  const expanded: number[] = [];
  const markdownViews: number[] = [];
  for (const line of text.split("\n")) {
    if (!line.includes("event=agent_chat_transcript_render")) continue;
    const elapsedMatch = line.match(/elapsed_ms=([0-9.]+)/);
    const previewMatch = line.match(/heavy_preview_count=(\d+)/);
    const expandedMatch = line.match(/expanded_heavy_markdown_count=(\d+)/);
    const markdownMatch = line.match(/markdown_view_count=(\d+)/);
    if (elapsedMatch) elapsed.push(Number(elapsedMatch[1]));
    if (previewMatch) heavyPreviews.push(Number(previewMatch[1]));
    if (expandedMatch) expanded.push(Number(expandedMatch[1]));
    if (markdownMatch) markdownViews.push(Number(markdownMatch[1]));
  }
  return {
    samples: elapsed.length,
    transcriptRenderAvgMs:
      elapsed.length > 0 ? Number((elapsed.reduce((a, b) => a + b, 0) / elapsed.length).toFixed(3)) : 0,
    transcriptRenderP95Ms: Number(percentile(elapsed, 0.95).toFixed(3)),
    transcriptRenderMaxMs: Number(Math.max(0, ...elapsed).toFixed(3)),
    heavyPreviewCountMin: heavyPreviews.length > 0 ? Math.min(...heavyPreviews) : 0,
    heavyPreviewCountMax: heavyPreviews.length > 0 ? Math.max(...heavyPreviews) : 0,
    expandedHeavyMarkdownCountMax: expanded.length > 0 ? Math.max(...expanded) : 0,
    fullMarkdownStateCountMaxDuringScroll: markdownViews.length > 0 ? Math.max(...markdownViews) : 0,
  };
}

function narrowScriptKitProcesses(): string[] {
  const proc = Bun.spawnSync({
    cmd: [
      "bash",
      "-lc",
      "ps -axo pid=,comm= | awk '$2 ~ /(^|\\/)script-kit-gpui$/ || $2 ~ /(^|\\/)app_mode_loader$/ || $2 ~ /(^|\\/)cargo-watch$/ {print}'",
    ],
    stdout: "pipe",
    stderr: "pipe",
  });
  return proc.stdout.toString().trim().split("\n").filter(Boolean);
}

function fail(receipt: Json, name: string, detail: Json = {}) {
  receipt.failures.push({ name, ...detail });
}

function within(value: number, expected: number, tolerance: number): boolean {
  return Math.abs(value - expected) <= tolerance;
}

function spread(values: number[]): number {
  if (values.length === 0) return 0;
  return Math.max(...values) - Math.min(...values);
}

async function sampleThumb(
  driver: Driver,
  label: string,
  itemIx: number,
  offsetPx = 0,
): Promise<Json> {
  const result = await driver.request(
    { type: "setAgentChatTranscriptScroll", itemIx, offsetPx },
    { expect: "externalCommandResult", timeoutMs: 5_000 },
  );
  await Bun.sleep(125);
  const state = await driver.request(
    { type: "getAgentChatState" },
    { expect: "agent_chatStateResult", timeoutMs: 10_000 },
  );
  return {
    label,
    requestedItemIx: itemIx,
    scrollCommandOk: result.ok !== false && result.success !== false,
    metrics: state.transcriptScroll ?? null,
  };
}

async function proveThumbGeometry(driver: Driver, rowCount: number): Promise<Json> {
  const middle = Math.floor(rowCount / 2);
  const requests = [
    ["topWarmup", 0],
    ["bottomWarmup", rowCount],
    ["top", 0],
    ["middle", middle],
    ["bottom", rowCount],
    ["topRepeat", 0],
    ["bottomRepeat", rowCount],
  ] as const;

  const samples: Json[] = [];
  for (const [label, itemIx] of requests) {
    samples.push(await sampleThumb(driver, label, itemIx));
  }

  const measured = samples.filter((sample) => sample.metrics != null && sample.scrollCommandOk);
  const required = ["top", "middle", "bottom", "topRepeat", "bottomRepeat"];
  const byLabel = new Map(samples.map((sample) => [sample.label, sample]));
  const requiredMetrics = required.map((label) => byLabel.get(label)?.metrics).filter(Boolean);
  const metricsPresent = requiredMetrics.length === required.length;

  const scrollTops = requiredMetrics.map((metrics) => metrics.scrollTopPx as number);
  const thumbTops = requiredMetrics.map((metrics) => metrics.thumbTopPx as number);
  const contentHeights = requiredMetrics.map((metrics) => metrics.contentHeightPx as number);
  const maxScrollTops = requiredMetrics.map((metrics) => metrics.maxScrollTopPx as number);
  const thumbHeights = requiredMetrics.map((metrics) => metrics.thumbHeightPx as number);

  const top = byLabel.get("top")?.metrics ?? null;
  const middleMetrics = byLabel.get("middle")?.metrics ?? null;
  const bottom = byLabel.get("bottom")?.metrics ?? null;
  const topRepeat = byLabel.get("topRepeat")?.metrics ?? null;
  const bottomRepeat = byLabel.get("bottomRepeat")?.metrics ?? null;

  const monotonicScrollTop =
    metricsPresent && top.scrollTopPx <= middleMetrics.scrollTopPx && middleMetrics.scrollTopPx <= bottom.scrollTopPx;
  const monotonicThumbTop =
    metricsPresent && top.thumbTopPx <= middleMetrics.thumbTopPx && middleMetrics.thumbTopPx <= bottom.thumbTopPx;
  const stableContentHeight = metricsPresent && spread(contentHeights) <= 2;
  const stableMaxScrollTop = metricsPresent && spread(maxScrollTops) <= 2;
  const stableThumbHeight = metricsPresent && spread(thumbHeights) <= 1;
  const topAnchored = metricsPresent && top.thumbTopPx <= 2 && top.scrollTopPx <= 2;
  const bottomAnchored =
    metricsPresent &&
    within(bottom.thumbBottomPx, bottom.thumbTrackHeightPx, 2) &&
    within(bottom.scrollTopPx, bottom.maxScrollTopPx, 2);
  const repeatedEndpointsStable =
    metricsPresent &&
    Math.abs(top.thumbTopPx - topRepeat.thumbTopPx) <= 2 &&
    Math.abs(top.scrollTopPx - topRepeat.scrollTopPx) <= 2 &&
    Math.abs(bottom.thumbBottomPx - bottomRepeat.thumbBottomPx) <= 2 &&
    Math.abs(bottom.scrollTopPx - bottomRepeat.scrollTopPx) <= 2;
  const thumbWithinTrack =
    metricsPresent &&
    requiredMetrics.every((metrics) => {
      const track = metrics.thumbTrackHeightPx as number;
      const height = metrics.thumbHeightPx as number;
      const topPx = metrics.thumbTopPx as number;
      const bottomPx = metrics.thumbBottomPx as number;
      return topPx >= -1 && topPx <= track - height + 1 && bottomPx >= height - 1 && bottomPx <= track + 1;
    });

  return {
    metricsPresent,
    measuredSamples: measured.length,
    canScrollY: metricsPresent && requiredMetrics.every((metrics) => metrics.canScrollY === true),
    monotonicScrollTop,
    monotonicThumbTop,
    stableContentHeight,
    stableMaxScrollTop,
    stableThumbHeight,
    topAnchored,
    bottomAnchored,
    repeatedEndpointsStable,
    thumbWithinTrack,
    samples,
  };
}

const args = parseArgs();
const receiptPath = resolve(args.receipt);
mkdirSync(dirname(receiptPath), { recursive: true });
mkdirSync(".test-output", { recursive: true });

const receipt: Json = {
  schemaVersion: 1,
  tool: "agent-chat-heavy-markdown-scroll-proof",
  classification: "blocked",
  pass: false,
  binary: args.binary,
  fixture: {
    messageCount: args.messageCount,
    assistantMarkdown: "heavy",
    heavyAssistantRowsExpected: Math.floor(args.messageCount / 2),
  },
  options: {
    proveThumb: args.proveThumb,
  },
  failures: [],
};

const driver = await Driver.launch({
  binary: args.binary,
  sandboxHome: true,
  sessionName: "agent-chat-heavy-scroll",
  defaultTimeoutMs: 10_000,
  env: {
    SCRIPT_KIT_AGENT_CHAT_RENDER_TRACE: "1",
    SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
  },
});

try {
  receipt.target = { pid: driver.pid, sessionDir: driver.sessionDir, logPath: driver.logPath };

  const opened = await driver.request(
    { type: "openAgentChatKitchenSinkFixture" },
    { expect: "externalCommandResult", timeoutMs: 10_000 },
  );
  receipt.openFixtureResult = opened;
  if (opened.ok === false || opened.success === false) {
    fail(receipt, "open_agent_chat_fixture_failed", { opened });
  }
  await Bun.sleep(1000);

  const fixture = await driver.request(
    {
      type: "setAgentChatTestFixture",
      phase: "idle",
      userText: userMarkdown(),
      assistantText: heavyAssistantMarkdown(),
      messageCount: args.messageCount,
    },
    { expect: "externalCommandResult", timeoutMs: 15_000 },
  );
  receipt.fixtureResult = fixture;
  if (fixture.ok === false || fixture.success === false) {
    fail(receipt, "set_fixture_failed", { fixture });
  }

  const state = await driver.request(
    { type: "getAgentChatState" },
    { expect: "agent_chatStateResult", timeoutMs: 10_000 },
  );
  receipt.fixture.stateMessageCount = state.messageCount;
  if (state.messageCount !== args.messageCount) {
    fail(receipt, "wrong_message_count", { state });
  }

  const windows = await driver.listAutomationWindows({ timeoutMs: 10_000 });
  const agentWindow = Array.isArray(windows.windows)
    ? windows.windows.find((win: Json) => win.semanticSurface === "agentChatChat" && win.visible === true) ??
      windows.windows.find((win: Json) => win.semanticSurface === "agentChatChat")
    : null;
  receipt.target = { ...receipt.target, window: agentWindow, windows };
  if (!agentWindow) {
    fail(receipt, "missing_agent_chat_window", { windows });
  }

  const positions = [0, 20, 40, 60, 80, 100, 120, 140, Math.max(0, args.messageCount - 1)];
  const ackMs: number[] = [];
  for (let i = 0; i < args.scrollCycles; i += 1) {
    const itemIx = positions[i % positions.length];
    const started = performance.now();
    const result = await driver.request(
      { type: "setAgentChatTranscriptScroll", itemIx, offsetPx: 0 },
      { expect: "externalCommandResult", timeoutMs: 5_000 },
    );
    ackMs.push(Math.round(performance.now() - started));
    if (result.ok === false || result.success === false) {
      fail(receipt, "scroll_command_failed", { i, itemIx, result });
      break;
    }
  }

  await Bun.sleep(400);

  const screenshotPaths = [
    ".test-output/agent-chat-heavy-scroll-top.png",
    ".test-output/agent-chat-heavy-scroll-middle.png",
    ".test-output/agent-chat-heavy-scroll-bottom.png",
  ];
  const screenshotTargets = [
    { type: "setAgentChatTranscriptScroll", itemIx: 0, offsetPx: 0 },
    { type: "setAgentChatTranscriptScroll", itemIx: Math.floor(args.messageCount / 2), offsetPx: 0 },
    { type: "setAgentChatTranscriptScroll", itemIx: Math.max(0, args.messageCount - 1), offsetPx: 0 },
  ];
  const screenshots: Json[] = [];
  for (let i = 0; i < screenshotTargets.length; i += 1) {
    await driver.request(screenshotTargets[i], { expect: "externalCommandResult", timeoutMs: 5_000 });
    await Bun.sleep(100);
    const shot = await driver.captureScreenshot({
      target: { type: "kind", kind: agentWindow?.kind ?? "ai" },
      savePath: screenshotPaths[i],
      timeoutMs: 15_000,
    });
    screenshots.push({ path: screenshotPaths[i], ok: !shot.error, width: shot.width, height: shot.height, error: shot.error });
  }

  const renderTrace = parseRenderTrace(driver.logPath);
  receipt.renderTrace = renderTrace;
  receipt.scrollStress = {
    cycles: args.scrollCycles,
    positionsVisited: positions,
    p95ScrollAckMs: percentile(ackMs, 0.95),
    maxScrollAckMs: Math.max(0, ...ackMs),
    slowCyclesOver100Ms: ackMs.filter((value) => value > 100).length,
  };
  receipt.visualProof = {
    screenshots,
    screenshotsTargetMatched: screenshots.every((shot) => shot.ok && shot.width > 0 && shot.height > 0),
  };
  if (args.proveThumb) {
    receipt.thumbProof = await proveThumbGeometry(driver, args.messageCount);
  }
  receipt.mitigation = {
    heavyPreviewRowsCollapsed: renderTrace.heavyPreviewCountMax,
    expandedHeavyMarkdownRows: renderTrace.expandedHeavyMarkdownCountMax,
    fullMarkdownStateCountMaxDuringScroll: renderTrace.fullMarkdownStateCountMaxDuringScroll,
  };

  if (renderTrace.heavyPreviewCountMax < Math.floor(args.messageCount / 2)) {
    fail(receipt, "heavy_previews_not_active", { renderTrace });
  }
  if (renderTrace.expandedHeavyMarkdownCountMax !== 0) {
    fail(receipt, "heavy_markdown_expanded_during_scroll", { renderTrace });
  }
  if (receipt.scrollStress.slowCyclesOver100Ms > 0) {
    fail(receipt, "slow_scroll_acks", { scrollStress: receipt.scrollStress });
  }
  if (!receipt.visualProof.screenshotsTargetMatched) {
    fail(receipt, "screenshot_failed", { visualProof: receipt.visualProof });
  }
  if (args.proveThumb) {
    const thumbProof = receipt.thumbProof ?? {};
    const failedThumbChecks = [
      "metricsPresent",
      "canScrollY",
      "monotonicScrollTop",
      "monotonicThumbTop",
      "stableContentHeight",
      "stableMaxScrollTop",
      "stableThumbHeight",
      "topAnchored",
      "bottomAnchored",
      "repeatedEndpointsStable",
      "thumbWithinTrack",
    ].filter((key) => thumbProof[key] !== true);
    if (failedThumbChecks.length > 0) {
      fail(receipt, "thumb_proof_failed", { failedThumbChecks, thumbProof });
    }
  }

  receipt.classification = receipt.failures.length === 0 ? "fixed" : "reproduced";
  receipt.pass = receipt.failures.length === 0;
} catch (error) {
  fail(receipt, "exception", { message: error instanceof Error ? error.message : String(error) });
  receipt.classification = "blocked";
} finally {
  await driver.close();
  await Bun.sleep(500);
  receipt.cleanup = {
    driverClosed: true,
    leftoverScriptKitProcesses: narrowScriptKitProcesses(),
  };
  if (receipt.cleanup.leftoverScriptKitProcesses.length > 0) {
    fail(receipt, "leftover_processes", receipt.cleanup);
    receipt.pass = false;
    receipt.classification = receipt.classification === "fixed" ? "blocked" : receipt.classification;
  }
  writeFileSync(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`);
  console.log(JSON.stringify(receipt, null, 2));
}

process.exit(receipt.pass ? 0 : 1);
