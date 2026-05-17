<script lang="ts">
  import AtlasBrief from "$lib/atlas-brief.svx";
  import { onMount } from "svelte";
  import data from "../data/features.generated.json";
  import type { Feature } from "../state/featureMachine";

  type VizMode =
    | "graph"
    | "signal"
    | "states"
    | "commands"
    | "workflows"
    | "risks"
    | "morphology"
    | "proof"
    | "stewardship"
    | "moats";

  type GraphNode = {
    feature: Feature;
    x: number;
    y: number;
    radius: number;
    clusterIndex: number;
  };

  type GraphEdge = {
    from: GraphNode;
    to: GraphNode;
    kind: "cluster" | "owner" | "selected";
  };

  type SignalColumn = {
    key: string;
    label: string;
    count: (feature: Feature) => number;
  };

  type BoundaryTag =
    | "privateLocalData"
    | "persistentStorage"
    | "permissions"
    | "clipboardOrSelection"
    | "filesAndPaths"
    | "screenshotsOrMedia"
    | "chatOrTranscript"
    | "automationProtocol"
    | "destructiveMutation"
    | "unknownBoundary";

  const features = data.features as Feature[];
  const coverage = data.coverage;
  const clusters = Array.from(new Set(features.map((feature) => feature.cluster ?? "Unclustered"))).sort();
  const owners = Array.from(
    new Set(features.flatMap((feature) => (feature.owners?.length ? feature.owners : [feature.owner ?? "Unowned"])))
  ).sort();

  const vizModes: { id: VizMode; label: string }[] = [
    { id: "graph", label: "Graph" },
    { id: "signal", label: "Signal Matrix" },
    { id: "states", label: "State Rails" },
    { id: "commands", label: "Command Grammar" },
    { id: "workflows", label: "Workflow Braids" },
    { id: "risks", label: "Risk Faultlines" },
    { id: "morphology", label: "Morphology Strip" },
    { id: "proof", label: "Proof Sonar" },
    { id: "stewardship", label: "Stewardship Delta" },
    { id: "moats", label: "Data Boundary Moats" }
  ];

  const signalColumns: SignalColumn[] = [
    { key: "workflows", label: "Workflows", count: (feature) => feature.workflows.length },
    { key: "interactions", label: "Inputs", count: (feature) => feature.interactions.length },
    { key: "states", label: "States", count: (feature) => feature.stateRows.length },
    { key: "keys", label: "Keys", count: (feature) => feature.keystrokes.length },
    { key: "risks", label: "Risks", count: (feature) => feature.risks.length },
    { key: "gaps", label: "Gaps", count: (feature) => feature.gaps.length },
    { key: "sections", label: "Sections", count: (feature) => Object.keys(feature.sections ?? {}).length },
    { key: "tables", label: "Tables", count: (feature) => Object.values(feature.tables ?? {}).filter((rows) => rows.length).length }
  ];

  const commandBuckets = [
    "Text",
    "Up",
    "Down",
    "Enter",
    "Tab",
    "Shift+Tab",
    "Escape",
    "Cmd+K",
    "Cmd+I",
    "Cmd+Y",
    "Cmd+Shift",
    "Click",
    "Scroll",
    "Other"
  ];
  const workflowPhases = ["Entry", "Filter", "Select", "Inspect", "Act", "Persist", "Return", "Verify", "Risk"];
  const riskThemes = [
    "selection",
    "focus",
    "actions",
    "protocol",
    "surface",
    "storage",
    "resize",
    "proof",
    "unsupported",
    "states",
    "config",
    "platform",
    "other"
  ];
  const morphologyLanes = ["mini-list", "split-preview", "grid", "popup", "protocol-api", "secondary-window", "storage-privacy"];
  const proofRings = [
    { id: "static", label: "Build/static", radius: 66, rx: /cargo check|build|fmt|source audit|contract/i },
    { id: "contract", label: "Unit/contract", radius: 104, rx: /test|contract|bun test|cargo test/i },
    { id: "runtime", label: "Runtime", radius: 142, rx: /agentic|waitFor|getState|getElements|runtime|receipt/i },
    { id: "visual", label: "Visual", radius: 180, rx: /screenshot|visual|chrome|storybook|pixel/i },
    { id: "protocol", label: "Protocol/storage", radius: 218, rx: /protocol|MCP|stdin|sqlite|storage|cache|resource/i }
  ];
  const boundaryMeta: { tag: BoundaryTag; label: string; weight: number; rx: RegExp }[] = [
    { tag: "permissions", label: "Permissions", weight: 5, rx: /permission|accessibility|screen recording|microphone|camera|AX/i },
    { tag: "privateLocalData", label: "Private data", weight: 5, rx: /privacy|private|local content|payload|redact|transcript|message|clipboard text/i },
    { tag: "destructiveMutation", label: "Destructive", weight: 4, rx: /delete|clear|remove|permanent|terminate|stop|kill|overwrite/i },
    { tag: "persistentStorage", label: "Storage", weight: 4, rx: /storage|sqlite|cache|history|persist|config\.ts|keychain|database/i },
    { tag: "screenshotsOrMedia", label: "Media", weight: 4, rx: /screenshot|image|audio|media|dictation|webcam|mic/i },
    { tag: "clipboardOrSelection", label: "Clipboard", weight: 3, rx: /clipboard|selected text|paste|copy/i },
    { tag: "filesAndPaths", label: "Files", weight: 3, rx: /file|path|directory|folder|attachment/i },
    { tag: "chatOrTranscript", label: "Chat", weight: 3, rx: /chat|conversation|message|ACP|AI|transcript/i },
    { tag: "automationProtocol", label: "Protocol", weight: 2, rx: /protocol|getState|getElements|receipt|MCP|stdin|automation|resource/i },
    { tag: "unknownBoundary", label: "Unknown", weight: 1, rx: /$a/ }
  ];

  let query = "";
  let selectedId = features[0]?.id ?? "";
  let focusCluster = "all";
  let focusOwner = "all";
  let relationMode: "cluster" | "owner" | "selected" = "selected";
  let vizMode: VizMode = "graph";
  let modeLens = "all";
  let groupMode: "cluster" | "owner" | "feature" = "cluster";

  onMount(() => {
    const requestedMode = new URLSearchParams(window.location.search).get("mode") as VizMode | null;
    if (requestedMode && vizModes.some((mode) => mode.id === requestedMode)) {
      vizMode = requestedMode;
    }
  });

  $: normalizedQuery = query.trim().toLowerCase();
  $: visibleFeatures = features.filter((feature) => {
    const ownerText = [feature.owner, ...(feature.owners ?? [])].join(" ");
    const text = [
      feature.id,
      feature.title,
      feature.indexTitle,
      feature.summary,
      feature.indexSummary,
      feature.cluster,
      ownerText,
      ...feature.capabilities,
      ...feature.risks,
      ...feature.gaps
    ]
      .filter(Boolean)
      .join(" ")
      .toLowerCase();
    const matchesQuery = !normalizedQuery || text.includes(normalizedQuery);
    const matchesCluster = focusCluster === "all" || feature.cluster === focusCluster;
    const matchesOwner =
      focusOwner === "all" || feature.owners?.includes(focusOwner) || feature.owner?.includes(focusOwner);
    return matchesQuery && matchesCluster && matchesOwner;
  });
  $: selectedFeature =
    visibleFeatures.find((feature) => feature.id === selectedId) ??
    features.find((feature) => feature.id === selectedId) ??
    visibleFeatures[0] ??
    features[0];
  $: graphNodes = layoutNodes(visibleFeatures);
  $: graphEdges = buildEdges(graphNodes, selectedFeature, relationMode);
  $: clusterCounts = clusters.map((cluster) => ({
    cluster,
    count: features.filter((feature) => feature.cluster === cluster).length
  }));
  $: signalMax = Object.fromEntries(
    signalColumns.map((column) => [column.key, Math.max(1, ...visibleFeatures.map((feature) => column.count(feature)))])
  ) as Record<string, number>;
  $: commandMap = visibleFeatures.map((feature) => ({ feature, buckets: commandBucketsFor(feature) }));
  $: workflowRows = visibleFeatures.map((feature) => ({ feature, threads: workflowThreads(feature) }));
  $: faultlineGroups = groupMode === "owner" ? owners : clusters;
  $: boundaryRows = boundaryMeta.map((meta) => ({ ...meta, features: visibleFeatures.filter((feature) => boundaryInfo(feature).tags.includes(meta.tag)) }));
  $: totalWorkflows = selectedFeature?.workflows.length ?? 0;
  $: totalSignals =
    (selectedFeature?.stateRows.length ?? 0) +
    (selectedFeature?.interactions.length ?? 0) +
    (selectedFeature?.keystrokes.length ?? 0);
  $: selectedBoundary = selectedFeature ? boundaryInfo(selectedFeature) : undefined;
  $: selectedProof = selectedFeature ? proofHits(selectedFeature) : [];

  function layoutNodes(items: Feature[]): GraphNode[] {
    const width = 920;
    const height = 560;
    const centerX = width / 2;
    const centerY = height / 2;
    const clusterByName = new Map(clusters.map((cluster, index) => [cluster, index]));
    const groups = new Map<string, Feature[]>();

    for (const feature of items) {
      const cluster = feature.cluster ?? "Unclustered";
      groups.set(cluster, [...(groups.get(cluster) ?? []), feature]);
    }

    const orderedGroups = Array.from(groups.entries()).sort(([a], [b]) => a.localeCompare(b));
    const nodes: GraphNode[] = [];
    orderedGroups.forEach(([cluster, group], groupIndex) => {
      const clusterAngle = (Math.PI * 2 * groupIndex) / Math.max(orderedGroups.length, 1) - Math.PI / 2;
      const groupRadius = orderedGroups.length > 1 ? 138 : 0;
      const groupCenterX = centerX + Math.cos(clusterAngle) * groupRadius;
      const groupCenterY = centerY + Math.sin(clusterAngle) * groupRadius;
      const localRadius = Math.max(24, Math.min(50, 18 + group.length * 2.4));

      group.forEach((feature, index) => {
        const angle = (Math.PI * 2 * index) / Math.max(group.length, 1) + groupIndex * 0.27;
        const weight = feature.workflows.length + feature.interactions.length + feature.stateRows.length;
        nodes.push({
          feature,
          x: groupCenterX + Math.cos(angle) * localRadius,
          y: groupCenterY + Math.sin(angle) * localRadius,
          radius: Math.max(11, Math.min(23, 9 + weight / 7)),
          clusterIndex: clusterByName.get(cluster) ?? 0
        });
      });
    });

    return nodes;
  }

  function buildEdges(nodes: GraphNode[], selected: Feature | undefined, mode: GraphEdge["kind"]): GraphEdge[] {
    const edges: GraphEdge[] = [];
    const byId = new Map(nodes.map((node) => [node.feature.id, node]));
    const selectedNode = selected ? byId.get(selected.id) : undefined;

    if (mode === "selected" && selectedNode) {
      const selectedOwners = new Set(selected?.owners ?? []);
      nodes.forEach((node) => {
        if (node === selectedNode) return;
        const sharesCluster = node.feature.cluster === selected?.cluster;
        const sharesOwner = (node.feature.owners ?? []).some((owner) => selectedOwners.has(owner));
        if (sharesCluster || sharesOwner) edges.push({ from: selectedNode, to: node, kind: "selected" });
      });
      return edges.slice(0, 28);
    }

    const groups = new Map<string, GraphNode[]>();
    nodes.forEach((node) => {
      const keys = mode === "owner" ? node.feature.owners ?? [] : [node.feature.cluster ?? "Unclustered"];
      keys.forEach((key) => groups.set(key, [...(groups.get(key) ?? []), node]));
    });

    for (const group of groups.values()) {
      group
        .sort((a, b) => a.feature.id.localeCompare(b.feature.id))
        .forEach((node, index) => {
          const next = group[index + 1];
          if (next) edges.push({ from: node, to: next, kind: mode });
        });
    }

    return edges.slice(0, 72);
  }

  function selectFeature(id: string) {
    selectedId = id;
  }

  function featureLabel(feature: Feature) {
    return (feature.indexTitle ?? feature.title).replace(/^\d+\s*/, "");
  }

  function featureText(feature: Feature, strict = false) {
    const sectionText = strict
      ? [
          feature.sections?.["Data, Storage, And Privacy Boundaries"],
          feature.sections?.["Automation And Protocol Surface"],
          feature.sections?.["Verification Recipes"]
        ]
      : [
          ...Object.values(feature.sections ?? {}),
          ...feature.capabilities,
          ...feature.workflows.map((workflow) => `${workflow.title} ${workflow.body}`),
          ...feature.interactions.flatMap((row) => Object.values(row)),
          ...feature.keystrokes.flatMap((row) => Object.values(row)),
          ...feature.risks,
          ...feature.gaps
        ];
    return sectionText.filter(Boolean).join("\n");
  }

  function tableCount(feature: Feature) {
    return Object.values(feature.tables ?? {}).reduce((count, rows) => count + rows.length, 0);
  }

  function commandBucketsFor(feature: Feature) {
    const buckets = new Set<string>();
    const rows = [...feature.keystrokes.flatMap((row) => Object.values(row)), ...feature.interactions.flatMap((row) => Object.values(row))];
    for (const value of rows) {
      const text = value.toLowerCase();
      if (/type|character|text|paste|query|filter/.test(text)) buckets.add("Text");
      if (/arrowup|arrow up|\bup\b/.test(text)) buckets.add("Up");
      if (/arrowdown|arrow down|\bdown\b/.test(text)) buckets.add("Down");
      if (/enter|return/.test(text)) buckets.add("Enter");
      if (/shift\+tab|shift tab/.test(text)) buckets.add("Shift+Tab");
      if (/\btab\b/.test(text)) buckets.add("Tab");
      if (/escape|esc/.test(text)) buckets.add("Escape");
      if (/cmd\+k|command\+k/.test(text)) buckets.add("Cmd+K");
      if (/cmd\+i|command\+i/.test(text)) buckets.add("Cmd+I");
      if (/cmd\+y|command\+y/.test(text)) buckets.add("Cmd+Y");
      if (/cmd\+shift|command\+shift/.test(text)) buckets.add("Cmd+Shift");
      if (/click|mouse|button/.test(text)) buckets.add("Click");
      if (/scroll|wheel/.test(text)) buckets.add("Scroll");
    }
    if (!buckets.size && rows.length) buckets.add("Other");
    return buckets;
  }

  function workflowThreads(feature: Feature) {
    return feature.workflows.slice(0, 6).map((workflow) => {
      const text = `${workflow.title} ${workflow.body}`.toLowerCase();
      return workflowPhases.filter((phase) => {
        const key = phase.toLowerCase();
        if (key === "entry") return /open|type|press|launch|entry|start/.test(text);
        if (key === "filter") return /filter|search|query|find/.test(text);
        if (key === "select") return /select|row|choose|arrow/.test(text);
        if (key === "inspect") return /preview|inspect|detail|info|read/.test(text);
        if (key === "act") return /run|execute|open|action|save|send|submit/.test(text);
        if (key === "persist") return /persist|storage|config|cache|write|history|save/.test(text);
        if (key === "return") return /return|close|escape|back|restore/.test(text);
        if (key === "verify") return /proof|verify|receipt|test|getstate|screenshot/.test(text);
        return /risk|error|disabled|empty|loading|gap/.test(text);
      });
    });
  }

  function classifyState(state: string) {
    const text = state.toLowerCase();
    if (/loading|searching|pending/.test(text)) return "loading";
    if (/error|disabled|empty|missing|fail/.test(text)) return "warning";
    if (/active|open|selected|focused|running/.test(text)) return "active";
    return "quiet";
  }

  function riskTheme(text: string) {
    const value = text.toLowerCase();
    if (/select|selected|row|stable/.test(value)) return "selection";
    if (/focus|keyboard|escape|enter|tab|cmd/.test(value)) return "focus";
    if (/action|popup|menu/.test(value)) return "actions";
    if (/protocol|getstate|receipt|automation|mcp/.test(value)) return "protocol";
    if (/surface|view|route|footer/.test(value)) return "surface";
    if (/storage|cache|sqlite|history|privacy|config/.test(value)) return "storage";
    if (/resize|size|layout|height|width|mini|full/.test(value)) return "resize";
    if (/proof|verify|test|coverage|receipt/.test(value)) return "proof";
    if (/unsupported|missing|gap|unknown/.test(value)) return "unsupported";
    if (/state|loading|empty|disabled/.test(value)) return "states";
    if (/config|preference|setting/.test(value)) return "config";
    if (/macos|window|appkit|screen|ax|platform/.test(value)) return "platform";
    return "other";
  }

  function inferMorphology(feature: Feature) {
    const text = featureText(feature).toLowerCase();
    if (/storage|sqlite|cache|history|privacy|config/.test(text)) return "storage-privacy";
    if (/protocol|getstate|getelements|mcp|stdin|api/.test(text)) return "protocol-api";
    if (/popup|dialog|modal|attached/.test(text)) return "popup";
    if (/secondary|detached|notes window|terminal|webcam|browser window/.test(text)) return "secondary-window";
    if (/grid|gallery|story|swatch/.test(text)) return "grid";
    if (/preview|details|inspector|split/.test(text)) return "split-preview";
    return "mini-list";
  }

  function proofHits(feature: Feature) {
    const text = feature.sections?.["Verification Recipes"] || featureText(feature);
    return proofRings.filter((ring) => ring.rx.test(text));
  }

  function boundaryInfo(feature: Feature) {
    const strictText = featureText(feature, true);
    const allText = `${strictText}\n${featureText(feature)}`;
    const matches = boundaryMeta
      .filter((meta) => meta.tag !== "unknownBoundary")
      .map((meta) => {
        const strictCount = (strictText.match(meta.rx) ?? []).length;
        const count = strictCount + (allText.match(meta.rx) ?? []).length;
        return { ...meta, count, strictCount, score: count * meta.weight };
      })
      .filter((match) => match.count > 0)
      .sort((a, b) => b.score - a.score);
    const tags = matches.length ? matches.map((match) => match.tag) : (["unknownBoundary"] as BoundaryTag[]);
    const strongest = matches[0]?.tag ?? "unknownBoundary";
    const score = matches.reduce((sum, match) => sum + match.score, 0);
    const evidence = [
      feature.sections?.["Data, Storage, And Privacy Boundaries"],
      feature.sections?.["Automation And Protocol Surface"],
      feature.risks[0],
      feature.gaps[0]
    ].filter(Boolean);
    return { tags, strongest, score, matches, evidence };
  }

  function groupKey(feature: Feature) {
    if (groupMode === "owner") return feature.owners?.[0] ?? feature.owner ?? "Unowned";
    if (groupMode === "feature") return feature.id;
    return feature.cluster ?? "Unclustered";
  }

  function pointForPolar(angle: number, radius: number, cx = 460, cy = 280) {
    return { x: cx + Math.cos(angle) * radius, y: cy + Math.sin(angle) * radius };
  }

  function selectedFeatureClass(feature: Feature) {
    return selectedFeature?.id === feature.id ? "is-selected" : "";
  }

  function totalWeight(feature: Feature) {
    return (
      feature.workflows.length +
      feature.interactions.length +
      feature.stateRows.length +
      feature.keystrokes.length +
      feature.risks.length +
      feature.gaps.length
    );
  }
</script>

<svelte:head>
  <title>Script Kit Feature Map Explorer</title>
</svelte:head>

<main class="experience">
  <section class="hero-panel" aria-label="Feature map controls">
    <div class="title-block">
      <p class="eyebrow">Script Kit feature map</p>
      <h1>Explore the atlas by relationship, proof, risk, ownership, and data boundary.</h1>
    </div>
    <div class="metrics" aria-label="Atlas coverage">
      <span><strong>{features.length}</strong> chapters</span>
      <span><strong>{coverage.rawOracleFeatureCount}</strong> Oracle slices</span>
      <span><strong>{clusters.length}</strong> clusters</span>
    </div>
    <div class="controls">
      <label>
        <span>Search</span>
        <input bind:value={query} placeholder="Feature, owner, workflow, risk" />
      </label>
      <label>
        <span>Cluster</span>
        <select bind:value={focusCluster}>
          <option value="all">All clusters</option>
          {#each clusters as cluster}
            <option value={cluster}>{cluster}</option>
          {/each}
        </select>
      </label>
      <label>
        <span>Owner</span>
        <select bind:value={focusOwner}>
          <option value="all">All owners</option>
          {#each owners as owner}
            <option value={owner}>{owner}</option>
          {/each}
        </select>
      </label>
      <div class="segmented" aria-label="Group mode">
        <button class:active={groupMode === "cluster"} on:click={() => (groupMode = "cluster")}>Cluster</button>
        <button class:active={groupMode === "owner"} on:click={() => (groupMode = "owner")}>Owner</button>
        <button class:active={groupMode === "feature"} on:click={() => (groupMode = "feature")}>Feature</button>
      </div>
    </div>
    <div class="viz-switcher" aria-label="Visualization mode">
      {#each vizModes as mode}
        <button
          class:active={vizMode === mode.id}
          on:click={() => {
            vizMode = mode.id;
            modeLens = "all";
          }}
        >
          {mode.label}
        </button>
      {/each}
    </div>
  </section>

  <section class="map-shell" aria-label="Interactive feature explorer">
    <div class="graph-wrap">
      <div class="mode-toolbar">
        <span>{vizModes.find((mode) => mode.id === vizMode)?.label}</span>
        {#if vizMode === "graph"}
          <div class="segmented compact" aria-label="Relationship mode">
            <button class:active={relationMode === "selected"} on:click={() => (relationMode = "selected")}>Selected</button>
            <button class:active={relationMode === "cluster"} on:click={() => (relationMode = "cluster")}>Cluster</button>
            <button class:active={relationMode === "owner"} on:click={() => (relationMode = "owner")}>Owner</button>
          </div>
        {:else if vizMode === "signal"}
          <div class="segmented compact"><button class:active={modeLens === "all"} on:click={() => (modeLens = "all")}>Coverage</button><button class:active={modeLens === "risks"} on:click={() => (modeLens = "risks")}>Risk</button><button class:active={modeLens === "gaps"} on:click={() => (modeLens = "gaps")}>Gaps</button></div>
        {:else if vizMode === "states"}
          <div class="segmented compact"><button class:active={modeLens === "all"} on:click={() => (modeLens = "all")}>States</button><button class:active={modeLens === "guards"} on:click={() => (modeLens = "guards")}>Guards</button><button class:active={modeLens === "risk"} on:click={() => (modeLens = "risk")}>Risk</button></div>
        {:else if vizMode === "commands"}
          <div class="segmented compact"><button class:active={modeLens === "all"} on:click={() => (modeLens = "all")}>All</button><button class:active={modeLens === "nav"} on:click={() => (modeLens = "nav")}>Navigation</button><button class:active={modeLens === "dismiss"} on:click={() => (modeLens = "dismiss")}>Dismiss</button></div>
        {:else if vizMode === "workflows"}
          <div class="segmented compact"><button class:active={modeLens === "all"} on:click={() => (modeLens = "all")}>All</button><button class:active={modeLens === "action"} on:click={() => (modeLens = "action")}>Action</button><button class:active={modeLens === "proof"} on:click={() => (modeLens = "proof")}>Proof</button></div>
        {:else if vizMode === "risks"}
          <div class="segmented compact"><button class:active={modeLens === "all"} on:click={() => (modeLens = "all")}>All</button><button class:active={modeLens === "risk"} on:click={() => (modeLens = "risk")}>Risks</button><button class:active={modeLens === "gap"} on:click={() => (modeLens = "gap")}>Gaps</button></div>
        {:else if vizMode === "moats"}
          <div class="segmented compact"><button class:active={modeLens === "all"} on:click={() => (modeLens = "all")}>All</button><button class:active={modeLens === "privacy"} on:click={() => (modeLens = "privacy")}>Privacy</button><button class:active={modeLens === "protocol"} on:click={() => (modeLens = "protocol")}>Protocol</button><button class:active={modeLens === "destructive"} on:click={() => (modeLens = "destructive")}>Destructive</button></div>
        {/if}
      </div>

      {#if vizMode === "graph"}
        <svg viewBox="0 0 920 560" role="img" aria-label="Interactive graph of feature map chapters">
          <defs>
            <filter id="nodeGlow" x="-60%" y="-60%" width="220%" height="220%">
              <feGaussianBlur stdDeviation="5" result="blur" />
              <feMerge>
                <feMergeNode in="blur" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
          </defs>
          {#each graphEdges as edge}
            <line
              class:edge-selected={edge.kind === "selected"}
              x1={edge.from.x}
              y1={edge.from.y}
              x2={edge.to.x}
              y2={edge.to.y}
            />
          {/each}
          {#each graphNodes as node}
            <g
              class:selected={selectedFeature?.id === node.feature.id}
              class="node node-cluster-{node.clusterIndex % 8}"
              transform="translate({node.x}, {node.y})"
              role="button"
              tabindex="0"
              on:click={() => selectFeature(node.feature.id)}
              on:keydown={(event) => {
                if (event.key === "Enter" || event.key === " ") selectFeature(node.feature.id);
              }}
            >
              <circle r={node.radius} />
              <text y="4">{node.feature.id}</text>
              <title>{featureLabel(node.feature)}</title>
            </g>
          {/each}
        </svg>
      {:else if vizMode === "signal"}
        <div class="viz-board matrix-board">
          <div class="matrix-head">
            <span>Feature</span>
            {#each signalColumns as column}
              <span>{column.label}</span>
            {/each}
          </div>
          {#each visibleFeatures as feature}
            <button class="matrix-row {selectedFeatureClass(feature)}" on:click={() => selectFeature(feature.id)}>
              <span><strong>{feature.id}</strong>{featureLabel(feature)}</span>
              {#each signalColumns as column}
                <i style={`--fill:${Math.max(4, (column.count(feature) / signalMax[column.key]) * 100)}%`}>
                  <b>{column.count(feature)}</b>
                </i>
              {/each}
            </button>
          {/each}
        </div>
      {:else if vizMode === "states"}
        <div class="viz-board rail-board">
          {#each visibleFeatures as feature}
            <button class="rail-row {selectedFeatureClass(feature)}" on:click={() => selectFeature(feature.id)}>
              <span><strong>{feature.id}</strong>{featureLabel(feature)}</span>
              <svg viewBox="0 0 680 54" aria-hidden="true">
                <line x1="12" y1="27" x2="668" y2="27" />
                {#each feature.stateRows.slice(0, 12) as row, index}
                  <circle class="state-{classifyState(row.State ?? Object.values(row)[0] ?? '')}" cx={24 + index * 54} cy="27" r={row.Guards ? 8 : 6} />
                {/each}
                {#each feature.interactions.slice(0, 10) as _, index}
                  <rect x={18 + index * 64} y="40" width="18" height="4" rx="2" />
                {/each}
                {#each feature.risks.slice(0, 6) as _, index}
                  <path d={`M${42 + index * 42} 10 l5 9 h-10 z`} />
                {/each}
              </svg>
            </button>
          {/each}
        </div>
      {:else if vizMode === "commands"}
        <div class="viz-board command-board">
          <div class="command-head">
            <span>Feature</span>
            {#each commandBuckets as bucket}
              <span>{bucket}</span>
            {/each}
          </div>
          {#each commandMap as row}
            <button class="command-row {selectedFeatureClass(row.feature)}" on:click={() => selectFeature(row.feature.id)}>
              <span><strong>{row.feature.id}</strong>{featureLabel(row.feature)}</span>
              {#each commandBuckets as bucket}
                <i class:filled={row.buckets.has(bucket)} title={bucket}></i>
              {/each}
            </button>
          {/each}
        </div>
      {:else if vizMode === "workflows"}
        <div class="viz-board braid-board">
          <div class="phase-head">
            <span></span>
            {#each workflowPhases as phase}
              <span>{phase}</span>
            {/each}
          </div>
          {#each workflowRows as row}
            <button class="braid-row {selectedFeatureClass(row.feature)}" on:click={() => selectFeature(row.feature.id)}>
              <span><strong>{row.feature.id}</strong>{featureLabel(row.feature)}</span>
              <svg viewBox="0 0 720 64" aria-hidden="true">
                {#each row.threads as thread, threadIndex}
                  {#if thread.length}
                    <polyline
                      points={thread
                        .map((phase) => `${36 + workflowPhases.indexOf(phase) * 78},${16 + threadIndex * 8}`)
                        .join(" ")}
                    />
                  {/if}
                {/each}
                {#each workflowPhases as phase, index}
                  <line x1={36 + index * 78} y1="8" x2={36 + index * 78} y2="56" />
                {/each}
              </svg>
            </button>
          {/each}
        </div>
      {:else if vizMode === "risks"}
        <div class="viz-board fault-board">
          <div class="fault-head">
            <span>Theme</span>
            {#each faultlineGroups.slice(0, 12) as group}
              <span>{group}</span>
            {/each}
          </div>
          {#each riskThemes as theme}
            <div class="fault-row">
              <span>{theme}</span>
              {#each faultlineGroups.slice(0, 12) as group}
                <div>
                  {#each visibleFeatures.filter((feature) => (groupMode === "owner" ? feature.owners?.includes(group) : feature.cluster === group) && [...feature.risks, ...feature.gaps].some((text) => riskTheme(text) === theme)).slice(0, 5) as feature}
                    <button class={selectedFeatureClass(feature)} on:click={() => selectFeature(feature.id)}>{feature.id}</button>
                  {/each}
                </div>
              {/each}
            </div>
          {/each}
        </div>
      {:else if vizMode === "morphology"}
        <svg class="viz-svg" viewBox="0 0 920 560" preserveAspectRatio="xMidYMin meet" role="img" aria-label="Surface Morphology Strip">
          {#each morphologyLanes as lane, laneIndex}
            <g class="morph-lane">
              <rect x="24" y={28 + laneIndex * 74} width="872" height="52" rx="7" />
              <text x="36" y={59 + laneIndex * 74}>{lane}</text>
            </g>
          {/each}
          {#each visibleFeatures as feature, index}
            {@const laneIndex = morphologyLanes.indexOf(inferMorphology(feature))}
            <g
              class="morph-glyph {selectedFeatureClass(feature)}"
              transform={`translate(${170 + (index % 22) * 32}, ${42 + laneIndex * 74})`}
              role="button"
              tabindex="0"
              on:click={() => selectFeature(feature.id)}
              on:keydown={(event) => {
                if (event.key === "Enter" || event.key === " ") selectFeature(feature.id);
              }}
            >
              <rect width={18 + Math.min(18, totalWeight(feature))} height={18 + Math.min(18, feature.interactions.length)} rx="4" />
              <text x="4" y="14">{feature.id.slice(-2)}</text>
            </g>
          {/each}
        </svg>
      {:else if vizMode === "proof"}
        <svg class="viz-svg" viewBox="0 0 920 560" preserveAspectRatio="xMidYMin meet" role="img" aria-label="Proof Sonar">
          {#each proofRings as ring}
            <circle class="sonar-ring" cx="460" cy="280" r={ring.radius} />
            <text x={466 + ring.radius} y="280">{ring.label}</text>
          {/each}
          {#each visibleFeatures as feature, index}
            {@const angle = (Math.PI * 2 * index) / Math.max(visibleFeatures.length, 1) - Math.PI / 2}
            {@const outer = pointForPolar(angle, 234)}
            <line class="sonar-spoke" x1="460" y1="280" x2={outer.x} y2={outer.y} />
            {#each proofHits(feature) as hit}
              {@const point = pointForPolar(angle, hit.radius)}
              <circle
                class="sonar-hit {selectedFeatureClass(feature)}"
                cx={point.x}
                cy={point.y}
                r={selectedFeature?.id === feature.id ? 7 : 4}
                role="button"
                tabindex="0"
                on:click={() => selectFeature(feature.id)}
                on:keydown={(event) => {
                  if (event.key === "Enter" || event.key === " ") selectFeature(feature.id);
                }}
              />
            {/each}
          {/each}
        </svg>
      {:else if vizMode === "stewardship"}
        <svg class="viz-svg" viewBox="0 0 920 560" preserveAspectRatio="xMidYMin meet" role="img" aria-label="Stewardship Delta">
          {#each owners.slice(0, 12) as owner, index}
            <text class="delta-label left" x="24" y={40 + index * 40}>{owner}</text>
          {/each}
          {#each clusters as cluster, index}
            <text class="delta-label" x="760" y={44 + index * 54}>{cluster}</text>
          {/each}
          {#each visibleFeatures as feature}
            {@const ownerIndex = Math.max(0, owners.indexOf(feature.owners?.[0] ?? feature.owner ?? ""))}
            {@const clusterIndex = Math.max(0, clusters.indexOf(feature.cluster ?? ""))}
            <path
              class="delta-ribbon {selectedFeatureClass(feature)}"
              d={`M210 ${38 + (ownerIndex % 12) * 40} C380 ${42 + (ownerIndex % 12) * 40}, 540 ${44 + clusterIndex * 54}, 736 ${44 + clusterIndex * 54}`}
              stroke-width={Math.max(1.2, Math.min(9, Math.sqrt(totalWeight(feature))))}
              role="button"
              tabindex="0"
              on:click={() => selectFeature(feature.id)}
              on:keydown={(event) => {
                if (event.key === "Enter" || event.key === " ") selectFeature(feature.id);
              }}
            />
          {/each}
        </svg>
      {:else if vizMode === "moats"}
        <svg class="viz-svg" viewBox="0 0 920 560" preserveAspectRatio="xMidYMin meet" role="img" aria-label="Data Boundary Moats">
          {#each boundaryRows as row, rowIndex}
            <g class="moat-band">
              <rect x="24" y={22 + rowIndex * 52} width="872" height="42" rx="6" />
              <text x="36" y={48 + rowIndex * 52}>{row.label}</text>
            </g>
            {#each row.features.slice(0, 26) as feature, index}
              {@const info = boundaryInfo(feature)}
              <circle
                class="moat-bead {selectedFeatureClass(feature)}"
                cx={178 + index * 27}
                cy={43 + rowIndex * 52}
                r={Math.max(7, Math.min(18, 7 + Math.sqrt(info.score)))}
                role="button"
                tabindex="0"
                on:click={() => selectFeature(feature.id)}
                on:keydown={(event) => {
                  if (event.key === "Enter" || event.key === " ") selectFeature(feature.id);
                }}
              />
            {/each}
          {/each}
        </svg>
      {/if}

      <div class="map-footer">
        <span>{visibleFeatures.length} visible</span>
        <span>{vizMode === "graph" ? `${graphEdges.length} relationships` : `${vizModes.length - 1} Oracle modes`}</span>
        <span>{selectedFeature?.cluster}</span>
      </div>
    </div>

    <aside class="inspector" aria-label="Selected feature details">
      {#if selectedFeature}
        <div class="inspector-heading">
          <p>{selectedFeature.file}</p>
          <h2>{featureLabel(selectedFeature)}</h2>
          <span>{selectedFeature.cluster}</span>
        </div>
        <div class="signal-row">
          <span><strong>{totalWorkflows}</strong> workflows</span>
          <span><strong>{totalSignals}</strong> state and interaction signals</span>
        </div>
        <p class="summary">{selectedFeature.indexSummary ?? selectedFeature.summary}</p>
        <div class="owner-list">
          {#each selectedFeature.owners ?? [] as owner}
            <button on:click={() => (focusOwner = owner)}>{owner}</button>
          {/each}
        </div>
        {#if vizMode === "moats" && selectedBoundary}
          <section class="detail-band">
            <h3>Boundary Evidence</h3>
            <div class="tag-row">
              {#each selectedBoundary.matches as match}
                <span>{match.label} {match.count}</span>
              {/each}
              {#if selectedBoundary.matches.length === 0}
                <span>Unknown boundary</span>
              {/if}
            </div>
            {#each selectedBoundary.evidence.slice(0, 2) as evidence}
              <p>{evidence}</p>
            {/each}
          </section>
        {:else if vizMode === "proof"}
          <section class="detail-band">
            <h3>Proof Rings</h3>
            <div class="tag-row">
              {#each selectedProof as proof}
                <span>{proof.label}</span>
              {/each}
            </div>
          </section>
        {/if}
        <section class="detail-band">
          <h3>Workflows</h3>
          {#each selectedFeature.workflows.slice(0, 4) as workflow}
            <button class="row-button" title={workflow.body}>{workflow.title}</button>
          {/each}
          {#if selectedFeature.workflows.length === 0}
            <p>No structured workflows found.</p>
          {/if}
        </section>
        <section class="detail-band">
          <h3>Risks</h3>
          <ul>
            {#each selectedFeature.risks.slice(0, 5) as risk}
              <li>{risk}</li>
            {/each}
          </ul>
        </section>
      {/if}
    </aside>
  </section>

  <section class="atlas-bottom" aria-label="Atlas guide and cluster index">
    <article class="mdx-panel">
      <AtlasBrief />
    </article>
    <div class="cluster-index">
      <h2>Cluster Index</h2>
      <div class="cluster-list">
        {#each clusterCounts as item}
          <button class:active={focusCluster === item.cluster} on:click={() => (focusCluster = item.cluster)}>
            <strong>{item.cluster}</strong>
            <span>{item.count}</span>
          </button>
        {/each}
      </div>
    </div>
  </section>
</main>
