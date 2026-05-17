<script lang="ts">
  import AtlasBrief from "$lib/atlas-brief.svx";
  import data from "../data/features.generated.json";
  import type { Feature } from "../state/featureMachine";

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

  const features = data.features as Feature[];
  const coverage = data.coverage;
  const clusters = Array.from(new Set(features.map((feature) => feature.cluster ?? "Unclustered"))).sort();
  const owners = Array.from(
    new Set(features.flatMap((feature) => feature.owners?.length ? feature.owners : [feature.owner ?? "Unowned"]))
  ).sort();

  let query = "";
  let selectedId = features[0]?.id ?? "";
  let focusCluster = "all";
  let focusOwner = "all";
  let relationMode: "cluster" | "owner" | "selected" = "selected";

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
      ...feature.capabilities
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
  $: selectedNode = graphNodes.find((node) => node.feature.id === selectedFeature?.id);
  $: clusterCounts = clusters.map((cluster) => ({
    cluster,
    count: features.filter((feature) => feature.cluster === cluster).length
  }));
  $: totalWorkflows = selectedFeature?.workflows.length ?? 0;
  $: totalSignals =
    (selectedFeature?.stateRows.length ?? 0) +
    (selectedFeature?.interactions.length ?? 0) +
    (selectedFeature?.keystrokes.length ?? 0);

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
</script>

<svelte:head>
  <title>Script Kit Feature Map Explorer</title>
</svelte:head>

<main class="experience">
  <section class="hero-panel" aria-label="Feature map controls">
    <div class="title-block">
      <p class="eyebrow">Script Kit feature map</p>
      <h1>Explore the atlas by relationship, not by reading order.</h1>
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
      <div class="segmented" aria-label="Relationship mode">
        <button class:active={relationMode === "selected"} on:click={() => (relationMode = "selected")}>Selected</button>
        <button class:active={relationMode === "cluster"} on:click={() => (relationMode = "cluster")}>Cluster</button>
        <button class:active={relationMode === "owner"} on:click={() => (relationMode = "owner")}>Owner</button>
      </div>
    </div>
  </section>

  <section class="map-shell" aria-label="Interactive feature graph">
    <div class="graph-wrap">
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
      <div class="map-footer">
        <span>{visibleFeatures.length} visible</span>
        <span>{graphEdges.length} relationships</span>
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
