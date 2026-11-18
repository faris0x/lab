<script lang="ts">
  import {
    CANVAS_HEIGHT,
    CANVAS_WIDTH,
    NODE_RADIUS,
    PALETTE,
    buildSvg,
    buildTikz,
    cloneGraph,
    downloadText,
    edgeGeometry,
    graphToJson,
    parseGraphJson,
    type DrawEdge,
    type DrawGraph,
    type DrawNode
  } from '../lib/graph-draw';

  type Mode = 'select' | 'node' | 'edge' | 'delete';
  type Selection = { kind: 'node'; id: string } | { kind: 'edge'; id: string } | null;

  const SAMPLE: DrawGraph = {
    title: 'Sample network',
    directed: false,
    weighted: true,
    nodes: [
      { id: 'n-a', label: 'A', x: 145, y: 145, color: '' },
      { id: 'n-b', label: 'B', x: 360, y: 100, color: '' },
      { id: 'n-c', label: 'C', x: 610, y: 165, color: '' },
      { id: 'n-d', label: 'D', x: 245, y: 370, color: '' },
      { id: 'n-e', label: 'E', x: 545, y: 385, color: '' }
    ],
    edges: [
      { id: 'e-ab', u: 'n-a', v: 'n-b', weight: '4', color: '' },
      { id: 'e-ad', u: 'n-a', v: 'n-d', weight: '2', color: '' },
      { id: 'e-bc', u: 'n-b', v: 'n-c', weight: '3', color: '' },
      { id: 'e-bd', u: 'n-b', v: 'n-d', weight: '5', color: '' },
      { id: 'e-ce', u: 'n-c', v: 'n-e', weight: '6', color: '' },
      { id: 'e-de', u: 'n-d', v: 'n-e', weight: '1', color: '' }
    ]
  };

  let graph = $state<DrawGraph>(cloneGraph(SAMPLE));
  let mode = $state<Mode>('select');
  let selected = $state<Selection>(null);
  let hovered = $state<Selection>(null);
  let edgeSource = $state<string | null>(null);
  let highlightNeighbours = $state(false);
  let showGrid = $state(true);
  let error = $state('');
  let notice = $state('');
  let past = $state<DrawGraph[]>([]);
  let future = $state<DrawGraph[]>([]);
  let svg: SVGSVGElement | undefined;
  let drag:
    | {
        id: string;
        startX: number;
        startY: number;
        nodeX: number;
        nodeY: number;
        snapshot: DrawGraph;
        moved: boolean;
      }
    | null = null;
  let sequence = 1;
  let noticeTimer: number | undefined;

  const selectedNode = $derived.by(() => {
    if (!selected || selected.kind !== 'node') return null;
    return graph.nodes.find((node) => node.id === selected!.id) ?? null;
  });
  const selectedEdge = $derived.by(() => {
    if (!selected || selected.kind !== 'edge') return null;
    return graph.edges.find((edge) => edge.id === selected!.id) ?? null;
  });
  const highlighted = $derived.by(() => {
    const nodes = new Set<string>();
    const edges = new Set<string>();
    if (!highlightNeighbours || !selected) return { nodes, edges };
    if (selected.kind === 'node') {
      nodes.add(selected.id);
      for (const edge of graph.edges) {
        if (edge.u === selected.id || edge.v === selected.id) {
          edges.add(edge.id);
          nodes.add(edge.u);
          nodes.add(edge.v);
        }
      }
    } else {
      const edge = graph.edges.find((item) => item.id === selected!.id);
      if (edge) {
        edges.add(edge.id);
        nodes.add(edge.u);
        nodes.add(edge.v);
      }
    }
    return { nodes, edges };
  });
  const dimming = $derived(highlightNeighbours && selected !== null);

  function newNodeId(): string {
    sequence += 1;
    return `node-${Date.now().toString(36)}-${sequence}`;
  }

  function newEdgeId(): string {
    sequence += 1;
    return `edge-${Date.now().toString(36)}-${sequence}`;
  }

  function nextLabel(existing: DrawNode[]): string {
    const used = new Set(existing.map((node) => node.label));
    for (let index = 0; index < 26; index += 1) {
      const label = String.fromCharCode(65 + index);
      if (!used.has(label)) return label;
    }
    let index = 1;
    while (used.has(`A${index}`)) index += 1;
    return `A${index}`;
  }

  function flash(message: string) {
    notice = message;
    if (noticeTimer) window.clearTimeout(noticeTimer);
    noticeTimer = window.setTimeout(() => (notice = ''), 2600);
  }

  function remember() {
    past = [...past.slice(-39), cloneGraph(graph)];
    future = [];
  }

  function syncSelection() {
    if (selected?.kind === 'node' && !graph.nodes.some((node) => node.id === selected!.id)) {
      selected = null;
    }
    if (selected?.kind === 'edge' && !graph.edges.some((edge) => edge.id === selected!.id)) {
      selected = null;
    }
    if (edgeSource && !graph.nodes.some((node) => node.id === edgeSource)) {
      edgeSource = null;
    }
  }

  function undo() {
    if (!past.length) return;
    const previous = past[past.length - 1];
    past = past.slice(0, -1);
    future = [cloneGraph(graph), ...future].slice(0, 40);
    graph = cloneGraph(previous);
    syncSelection();
  }

  function redo() {
    if (!future.length) return;
    const next = future[0];
    future = future.slice(1);
    past = [...past, cloneGraph(graph)].slice(-40);
    graph = cloneGraph(next);
    syncSelection();
  }

  function svgPoint(event: PointerEvent): { x: number; y: number } {
    if (!svg) return { x: 0, y: 0 };
    const rect = svg.getBoundingClientRect();
    return {
      x: ((event.clientX - rect.left) / rect.width) * CANVAS_WIDTH,
      y: ((event.clientY - rect.top) / rect.height) * CANVAS_HEIGHT
    };
  }

  function clamp(value: number, min: number, max: number): number {
    return Math.max(min, Math.min(max, value));
  }

  function addNodeAt(x: number, y: number) {
    if (graph.nodes.length >= 32) {
      error = 'Use at most 32 vertices.';
      return;
    }
    remember();
    const node: DrawNode = {
      id: newNodeId(),
      label: nextLabel(graph.nodes),
      x: clamp(x, NODE_RADIUS + 10, CANVAS_WIDTH - NODE_RADIUS - 10),
      y: clamp(y, 50, CANVAS_HEIGHT - NODE_RADIUS - 10),
      color: ''
    };
    graph.nodes = [...graph.nodes, node];
    selected = { kind: 'node', id: node.id };
    error = '';
  }

  function addEdge(u: string, v: string) {
    if (u === v) {
      error = 'A vertex cannot be joined to itself.';
      return;
    }
    const existing = graph.edges.find(
      (edge) =>
        (edge.u === u && edge.v === v) ||
        (!graph.directed && edge.u === v && edge.v === u)
    );
    if (existing) {
      selected = { kind: 'edge', id: existing.id };
      error = 'That edge already exists.';
      edgeSource = null;
      return;
    }
    if (graph.edges.length >= 80) {
      error = 'Use at most 80 edges.';
      return;
    }
    remember();
    const edge: DrawEdge = {
      id: newEdgeId(),
      u,
      v,
      weight: '1',
      color: ''
    };
    graph.edges = [...graph.edges, edge];
    selected = { kind: 'edge', id: edge.id };
    edgeSource = null;
    error = '';
  }

  function deleteNode(id: string) {
    remember();
    graph.nodes = graph.nodes.filter((node) => node.id !== id);
    graph.edges = graph.edges.filter((edge) => edge.u !== id && edge.v !== id);
    if (selected?.kind === 'node' && selected.id === id) selected = null;
    if (edgeSource === id) edgeSource = null;
    syncSelection();
  }

  function deleteEdge(id: string) {
    remember();
    graph.edges = graph.edges.filter((edge) => edge.id !== id);
    if (selected?.kind === 'edge' && selected.id === id) selected = null;
  }

  function deleteSelection() {
    if (selected?.kind === 'node') deleteNode(selected.id);
    else if (selected?.kind === 'edge') deleteEdge(selected.id);
  }

  function onCanvasPointerDown(event: PointerEvent) {
    const point = svgPoint(event);
    if (mode === 'node') {
      addNodeAt(point.x, point.y);
    } else if (mode === 'select') {
      selected = null;
      edgeSource = null;
    } else if (mode === 'edge') {
      edgeSource = null;
      selected = null;
    }
  }

  function onNodePointerDown(event: PointerEvent, node: DrawNode) {
    event.stopPropagation();
    if (mode === 'delete') {
      deleteNode(node.id);
      return;
    }
    if (mode === 'edge') {
      if (!edgeSource) {
        edgeSource = node.id;
        selected = { kind: 'node', id: node.id };
      } else if (edgeSource !== node.id) {
        addEdge(edgeSource, node.id);
      }
      return;
    }
    if (mode === 'node') return;
    selected = { kind: 'node', id: node.id };
    const point = svgPoint(event);
    drag = {
      id: node.id,
      startX: point.x,
      startY: point.y,
      nodeX: node.x,
      nodeY: node.y,
      snapshot: cloneGraph(graph),
      moved: false
    };
  }

  function onEdgePointerDown(event: PointerEvent, edge: DrawEdge) {
    event.stopPropagation();
    if (mode === 'delete') {
      deleteEdge(edge.id);
      return;
    }
    if (mode === 'select') {
      selected = { kind: 'edge', id: edge.id };
      edgeSource = null;
    }
  }

  function onPointerMove(event: PointerEvent) {
    if (!drag) return;
    const point = svgPoint(event);
    const dx = point.x - drag.startX;
    const dy = point.y - drag.startY;
    if (!drag.moved && Math.hypot(dx, dy) > 2) {
      past = [...past.slice(-39), drag.snapshot];
      future = [];
      drag.moved = true;
    }
    if (!drag.moved) return;
    graph.nodes = graph.nodes.map((node) =>
      node.id === drag?.id
        ? {
            ...node,
            x: clamp(drag.nodeX + dx, NODE_RADIUS + 6, CANVAS_WIDTH - NODE_RADIUS - 6),
            y: clamp(drag.nodeY + dy, 44, CANVAS_HEIGHT - NODE_RADIUS - 6)
          }
        : node
    );
  }

  function onPointerUp() {
    drag = null;
  }

  function handleKeydown(event: KeyboardEvent) {
    const target = event.target as HTMLElement | null;
    if (
      target &&
      (target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.tagName === 'SELECT' ||
        target.isContentEditable)
    ) {
      return;
    }
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'z') {
      event.preventDefault();
      if (event.shiftKey) redo();
      else undo();
      return;
    }
    if (event.key === 'Escape') {
      selected = null;
      edgeSource = null;
      error = '';
      return;
    }
    if (event.key === 'Delete' || event.key === 'Backspace') {
      if (selected) {
        event.preventDefault();
        deleteSelection();
      }
    }
  }

  function tidyLayout() {
    const count = graph.nodes.length;
    if (!count) return;
    remember();
    const radiusX = Math.min(290, Math.max(130, count * 34));
    const radiusY = Math.min(190, Math.max(100, count * 24));
    graph.nodes = graph.nodes.map((node, index) => {
      const angle = -Math.PI / 2 + (index * Math.PI * 2) / count;
      return {
        ...node,
        x: CANVAS_WIDTH / 2 + radiusX * Math.cos(angle),
        y: CANVAS_HEIGHT / 2 + radiusY * Math.sin(angle)
      };
    });
  }

  function loadSample() {
    remember();
    graph = cloneGraph(SAMPLE);
    selected = null;
    edgeSource = null;
    mode = 'select';
    error = '';
  }

  function clearGraph() {
    remember();
    graph.nodes = [];
    graph.edges = [];
    selected = null;
    edgeSource = null;
    error = '';
  }

  function setWeighted(value: boolean) {
    remember();
    graph.weighted = value;
  }

  function setDirected(value: boolean) {
    remember();
    graph.directed = value;
  }

  function setNodeLabel(node: DrawNode, value: string) {
    graph.nodes = graph.nodes.map((item) =>
      item.id === node.id ? { ...item, label: value.slice(0, 8) } : item
    );
  }

  function setEdgeWeight(edge: DrawEdge, value: string) {
    graph.edges = graph.edges.map((item) =>
      item.id === edge.id ? { ...item, weight: value.slice(0, 12) } : item
    );
  }

  function setNodeColor(node: DrawNode, color: string) {
    remember();
    graph.nodes = graph.nodes.map((item) =>
      item.id === node.id ? { ...item, color } : item
    );
  }

  function setEdgeColor(edge: DrawEdge, color: string) {
    remember();
    graph.edges = graph.edges.map((item) =>
      item.id === edge.id ? { ...item, color } : item
    );
  }

  function reverseEdge(edge: DrawEdge) {
    remember();
    graph.edges = graph.edges.map((item) =>
      item.id === edge.id ? { ...item, u: item.v, v: item.u } : item
    );
  }

  function fileBase(): string {
    return (
      graph.title
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, '-')
        .replace(/^-|-$/g, '') || 'graph'
    );
  }

  function exportSvg() {
    downloadText(`${fileBase()}.svg`, buildSvg(graph), 'image/svg+xml');
    flash('SVG downloaded.');
  }

  function exportTikz() {
    downloadText(`${fileBase()}.tex`, buildTikz(graph), 'application/x-tex');
    flash('TikZ downloaded.');
  }

  function exportJson() {
    downloadText(`${fileBase()}.json`, graphToJson(graph), 'application/json');
    flash('Graph JSON downloaded.');
  }

  async function importJson(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    try {
      const text = await file.text();
      const parsed = parseGraphJson(text);
      remember();
      graph = parsed;
      selected = null;
      edgeSource = null;
      error = '';
      flash('Graph imported.');
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      input.value = '';
    }
  }

  async function copySvg() {
    try {
      await navigator.clipboard.writeText(buildSvg(graph));
      flash('SVG copied.');
    } catch {
      error = 'Could not copy the SVG in this browser.';
    }
  }

  function onNodeKeydown(event: KeyboardEvent, node: DrawNode) {
    if (event.key !== 'Enter' && event.key !== ' ') return;
    event.preventDefault();
    if (mode === 'delete') {
      deleteNode(node.id);
    } else if (mode === 'edge') {
      if (!edgeSource) {
        edgeSource = node.id;
        selected = { kind: 'node', id: node.id };
      } else if (edgeSource !== node.id) {
        addEdge(edgeSource, node.id);
      }
    } else {
      selected = { kind: 'node', id: node.id };
    }
  }

  function onEdgeKeydown(event: KeyboardEvent, edge: DrawEdge) {
    if (event.key !== 'Enter' && event.key !== ' ') return;
    event.preventDefault();
    if (mode === 'delete') {
      deleteEdge(edge.id);
    } else {
      selected = { kind: 'edge', id: edge.id };
      edgeSource = null;
    }
  }

  function nodeDimmed(id: string): boolean {
    return dimming && !highlighted.nodes.has(id);
  }

  function edgeDimmed(id: string): boolean {
    return dimming && !highlighted.edges.has(id);
  }

  function nodeById(id: string): DrawNode | undefined {
    return graph.nodes.find((node) => node.id === id);
  }
</script>

<svelte:window onpointermove={onPointerMove} onpointerup={onPointerUp} onkeydown={handleKeydown} />

<div class="draw-shell">
  <div class="draw-toolbar" role="toolbar" aria-label="Graph drawing tools">
    <div class="btn-row">
      <button
        class="btn btn-small"
        class:btn-active={mode === 'select'}
        type="button"
        aria-pressed={mode === 'select'}
        onclick={() => (mode = 'select')}>Select / move</button
      >
      <button
        class="btn btn-small"
        class:btn-active={mode === 'node'}
        type="button"
        aria-pressed={mode === 'node'}
        onclick={() => (mode = 'node')}>Add vertex</button
      >
      <button
        class="btn btn-small"
        class:btn-active={mode === 'edge'}
        type="button"
        aria-pressed={mode === 'edge'}
        onclick={() => {
          mode = 'edge';
          edgeSource = null;
        }}>Add edge</button
      >
      <button
        class="btn btn-small"
        class:btn-active={mode === 'delete'}
        type="button"
        aria-pressed={mode === 'delete'}
        onclick={() => (mode = 'delete')}>Delete</button
      >
      <button class="btn btn-ghost btn-small" type="button" onclick={undo} disabled={!past.length}
        >Undo</button
      >
      <button class="btn btn-ghost btn-small" type="button" onclick={redo} disabled={!future.length}
        >Redo</button
      >
      <button class="btn btn-ghost btn-small" type="button" onclick={tidyLayout}>Tidy layout</button>
      <button class="btn btn-ghost btn-small" type="button" onclick={loadSample}>Sample</button>
      <button class="btn btn-ghost btn-small" type="button" onclick={clearGraph}>Clear</button>
    </div>
  </div>

  <div class="draw-layout">
    <div class="draw-canvas" class:mode-node={mode === 'node'} class:mode-edge={mode === 'edge'} class:mode-delete={mode === 'delete'}>
      <svg
        bind:this={svg}
        viewBox={`0 0 ${CANVAS_WIDTH} ${CANVAS_HEIGHT}`}
        role="img"
        aria-label="Graph drawing canvas"
        onpointerdown={onCanvasPointerDown}
      >
        <defs>
          <pattern id="draw-grid" width="32" height="32" patternUnits="userSpaceOnUse">
            <path d="M32 0H0V32" fill="none" stroke="var(--line-soft)" stroke-width="0.7" />
          </pattern>
        </defs>
        <rect class="canvas-bg" width={CANVAS_WIDTH} height={CANVAS_HEIGHT} />
        {#if showGrid}
          <rect width={CANVAS_WIDTH} height={CANVAS_HEIGHT} fill="url(#draw-grid)" />
        {/if}

        {#each graph.edges as edge (edge.id)}
          {@const from = nodeById(edge.u)}
          {@const to = nodeById(edge.v)}
          {#if from && to}
            {@const geometry = edgeGeometry(from, to, graph.directed)}
            <g
              class="draw-edge"
              class:is-selected={selected?.kind === 'edge' && selected.id === edge.id}
              class:is-hovered={hovered?.kind === 'edge' && hovered.id === edge.id}
              class:is-source={edgeSource === edge.u}
              class:is-dimmed={edgeDimmed(edge.id)}
            >
              <line
                class="edge-hit"
                x1={geometry.x1}
                y1={geometry.y1}
                x2={geometry.x2}
                y2={geometry.y2}
                role="button"
                tabindex="0"
                aria-label={`Edge ${from.label} to ${to.label}`}
                onpointerdown={(event) => onEdgePointerDown(event, edge)}
                onpointerenter={() => (hovered = { kind: 'edge', id: edge.id })}
                onpointerleave={() => (hovered = null)}
                onkeydown={(event) => onEdgeKeydown(event, edge)}
              />
              <line
                class="edge-line"
                x1={geometry.x1}
                y1={geometry.y1}
                x2={geometry.x2}
                y2={geometry.y2}
                stroke={selected?.kind === 'edge' && selected.id === edge.id
                  ? 'var(--accent)'
                  : edge.color || undefined}
              />
              {#if graph.directed}
                <polygon
                  class="edge-arrow"
                  points={geometry.arrow}
                  fill={edge.color || 'var(--fg-faint)'}
                />
              {/if}
              {#if graph.weighted && edge.weight.trim()}
                <text
                  class="edge-weight"
                  x={geometry.labelX}
                  y={geometry.labelY}
                  text-anchor="middle"
                  dominant-baseline="middle"
                  >{edge.weight}</text
                >
              {/if}
            </g>
          {/if}
        {/each}

        {#each graph.nodes as node (node.id)}
          <g
            class="draw-node"
            class:is-selected={selected?.kind === 'node' && selected.id === node.id}
            class:is-hovered={hovered?.kind === 'node' && hovered.id === node.id}
            class:is-source={edgeSource === node.id}
            class:is-dimmed={nodeDimmed(node.id)}
            role="button"
            tabindex="0"
            aria-label={`Vertex ${node.label}`}
            onpointerdown={(event) => onNodePointerDown(event, node)}
            onpointerenter={() => (hovered = { kind: 'node', id: node.id })}
            onpointerleave={() => (hovered = null)}
            onkeydown={(event) => onNodeKeydown(event, node)}
          >
            <circle
              class="node-shape"
              cx={node.x}
              cy={node.y}
              r={NODE_RADIUS}
              fill={node.color || undefined}
              stroke={selected?.kind === 'node' && selected.id === node.id
                ? '#f4ead1'
                : node.color
                  ? '#f4ead1'
                  : undefined}
            />
            <text class="node-label" x={node.x} y={node.y} text-anchor="middle" dominant-baseline="middle"
              >{node.label}</text
            >
          </g>
        {/each}

        {#if graph.nodes.length === 0}
          <text class="empty-canvas" x={CANVAS_WIDTH / 2} y={CANVAS_HEIGHT / 2} text-anchor="middle">
            Choose “Add vertex” and click the canvas.
          </text>
        {/if}
      </svg>
      {#if notice}<p class="draw-notice">{notice}</p>{/if}
    </div>

    <aside class="draw-inspector">
      <label class="field">
        <span>Graph title</span>
        <input value={graph.title} oninput={(event) => (graph.title = event.currentTarget.value)} />
      </label>

      <div class="draw-toggles">
        <label><input type="checkbox" checked={graph.weighted} onchange={(event) => setWeighted(event.currentTarget.checked)} /> Weighted</label>
        <label><input type="checkbox" checked={graph.directed} onchange={(event) => setDirected(event.currentTarget.checked)} /> Directed</label>
        <label><input type="checkbox" checked={showGrid} onchange={(event) => (showGrid = event.currentTarget.checked)} /> Grid</label>
        <label><input type="checkbox" checked={highlightNeighbours} onchange={(event) => (highlightNeighbours = event.currentTarget.checked)} /> Highlight neighbours</label>
      </div>

      {#if selectedNode}
        <div class="draw-selection">
          <h3>Vertex {selectedNode.label}</h3>
          <label class="field">
            <span>Label</span>
            <input
              value={selectedNode.label}
              oninput={(event) => setNodeLabel(selectedNode, event.currentTarget.value)}
            />
          </label>
          <p class="draw-subhead">Highlight colour</p>
          <div class="swatches">
            {#each PALETTE as colour}
              <button
                type="button"
                class="swatch"
                class:active={selectedNode.color === colour.value}
                style={`--swatch:${colour.value}`}
                aria-label={`Colour ${colour.label}`}
                onclick={() => setNodeColor(selectedNode, colour.value)}
              ></button>
            {/each}
            <button type="button" class="btn btn-ghost btn-small" onclick={() => setNodeColor(selectedNode, '')}
              >Clear colour</button
            >
          </div>
          <button class="btn btn-ghost btn-small" type="button" onclick={() => deleteNode(selectedNode.id)}
            >Delete vertex</button
          >
        </div>
      {:else if selectedEdge}
        <div class="draw-selection">
          <h3>Edge {nodeById(selectedEdge.u)?.label ?? '?'}–{nodeById(selectedEdge.v)?.label ?? '?'}</h3>
          {#if graph.weighted}
            <label class="field">
              <span>Weight</span>
              <input
                value={selectedEdge.weight}
                oninput={(event) => setEdgeWeight(selectedEdge, event.currentTarget.value)}
              />
            </label>
          {/if}
          <p class="draw-subhead">Highlight colour</p>
          <div class="swatches">
            {#each PALETTE as colour}
              <button
                type="button"
                class="swatch"
                class:active={selectedEdge.color === colour.value}
                style={`--swatch:${colour.value}`}
                aria-label={`Colour ${colour.label}`}
                onclick={() => setEdgeColor(selectedEdge, colour.value)}
              ></button>
            {/each}
            <button type="button" class="btn btn-ghost btn-small" onclick={() => setEdgeColor(selectedEdge, '')}
              >Clear colour</button
            >
          </div>
          <div class="btn-row">
            {#if graph.directed}
              <button class="btn btn-ghost btn-small" type="button" onclick={() => reverseEdge(selectedEdge)}
                >Reverse</button
              >
            {/if}
            <button class="btn btn-ghost btn-small" type="button" onclick={() => deleteEdge(selectedEdge.id)}
              >Delete edge</button
            >
          </div>
        </div>
      {:else}
        <div class="draw-help">
          <p><strong>Select / move</strong> drags vertices.</p>
          <p><strong>Add vertex</strong> clicks the canvas.</p>
          <p><strong>Add edge</strong> clicks two vertices.</p>
          <p><strong>Delete</strong> removes a vertex or edge.</p>
        </div>
      {/if}

      <div class="draw-export">
        <p class="draw-subhead">Export</p>
        <div class="btn-row">
          <button class="btn btn-ghost btn-small" type="button" onclick={exportSvg}>SVG</button>
          <button class="btn btn-ghost btn-small" type="button" onclick={copySvg}>Copy SVG</button>
          <button class="btn btn-ghost btn-small" type="button" onclick={exportTikz}>TikZ</button>
          <button class="btn btn-ghost btn-small" type="button" onclick={exportJson}>JSON</button>
          <label class="btn btn-ghost btn-small file-label">
            Import JSON
            <input type="file" accept="application/json,.json" onchange={importJson} />
          </label>
        </div>
      </div>

      {#if error}<p class="error" role="alert">{error}</p>{/if}
    </aside>
  </div>
</div>

<style>
  .draw-shell {
    display: grid;
    gap: 0.8rem;
  }

  .draw-toolbar {
    border: 1px solid var(--line);
    border-radius: var(--radius);
    padding: 0.55rem 0.65rem;
    background: var(--bg-soft);
  }

  .btn-active {
    border-color: var(--accent);
    color: var(--accent);
    background: var(--accent-soft);
  }

  .draw-layout {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 18rem;
    gap: 0.9rem;
    align-items: start;
  }

  .draw-canvas {
    position: relative;
    border: 1px solid var(--line);
    border-radius: var(--radius);
    overflow: hidden;
    background: var(--bg-card);
  }

  .draw-canvas svg {
    display: block;
    width: 100%;
    height: auto;
    touch-action: none;
    user-select: none;
    cursor: default;
  }

  .draw-canvas.mode-node svg {
    cursor: crosshair;
  }

  .draw-canvas.mode-edge svg {
    cursor: cell;
  }

  .draw-canvas.mode-delete svg {
    cursor: not-allowed;
  }

  .canvas-bg {
    fill: var(--bg-card);
  }

  .draw-node {
    cursor: grab;
  }

  .draw-node:active {
    cursor: grabbing;
  }

  .node-shape {
    fill: var(--bg-card);
    stroke: var(--accent);
    stroke-width: 2;
  }

  .node-label {
    fill: var(--fg);
    stroke: none;
    font-family: var(--sans);
    font-size: 15px;
    font-weight: 700;
    pointer-events: none;
  }

  .draw-node.is-selected .node-shape {
    stroke: #ffffff !important;
    stroke-width: 3;
  }

  .draw-node.is-hovered .node-shape {
    filter: drop-shadow(0 0 6px var(--accent));
  }

  .draw-node.is-source .node-shape {
    stroke: var(--warn);
    stroke-width: 4;
  }

  .draw-node.is-dimmed,
  .draw-edge.is-dimmed {
    opacity: 0.25;
  }

  .edge-hit {
    stroke: transparent;
    stroke-width: 18;
    pointer-events: stroke;
    cursor: pointer;
  }

  .edge-line {
    stroke: var(--fg-faint);
    stroke-width: 2.4;
    stroke-linecap: round;
    pointer-events: none;
  }

  .draw-edge.is-selected .edge-line {
    stroke: var(--accent) !important;
    stroke-width: 3.5;
  }

  .draw-edge.is-hovered .edge-line {
    stroke: var(--fg);
    filter: drop-shadow(0 0 4px var(--accent));
  }

  .edge-arrow {
    pointer-events: none;
  }

  .edge-weight {
    fill: var(--fg);
    stroke: var(--bg-card);
    stroke-width: 4;
    paint-order: stroke;
    font-size: 13px;
    font-weight: 600;
    pointer-events: none;
  }

  .empty-canvas {
    fill: var(--fg-faint);
    font-family: var(--sans);
    font-size: 16px;
  }

  .draw-notice {
    position: absolute;
    right: 0.75rem;
    bottom: 0.65rem;
    margin: 0;
    border: 1px solid var(--line);
    border-radius: 6px;
    padding: 0.25rem 0.5rem;
    background: var(--bg-soft);
    color: var(--fg-dim);
    font-size: 0.76rem;
  }

  .draw-inspector {
    border: 1px solid var(--line);
    border-radius: var(--radius);
    padding: 0.85rem;
    background: var(--bg-soft);
    display: grid;
    gap: 0.8rem;
  }

  .draw-toggles {
    display: grid;
    gap: 0.35rem;
    color: var(--fg-dim);
    font-size: 0.84rem;
  }

  .draw-toggles label {
    display: flex;
    align-items: center;
    gap: 0.45rem;
  }

  .draw-toggles input {
    width: auto;
  }

  .draw-selection {
    border-top: 1px solid var(--line-soft);
    padding-top: 0.75rem;
    display: grid;
    gap: 0.65rem;
  }

  .draw-selection h3 {
    margin: 0;
    font-family: var(--mono);
    font-size: 0.78rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--accent);
  }

  .draw-subhead {
    margin: 0;
    color: var(--fg-faint);
    font-family: var(--mono);
    font-size: 0.68rem;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  .swatches {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
  }

  .swatch {
    width: 1.55rem;
    height: 1.55rem;
    border: 2px solid transparent;
    border-radius: 50%;
    background: var(--swatch);
    cursor: pointer;
    padding: 0;
  }

  .swatch.active {
    border-color: var(--fg);
  }

  .draw-help {
    border-top: 1px solid var(--line-soft);
    padding-top: 0.65rem;
    color: var(--fg-dim);
    font-size: 0.84rem;
  }

  .draw-help p {
    margin: 0.3rem 0;
  }

  .draw-export {
    border-top: 1px solid var(--line-soft);
    padding-top: 0.7rem;
    display: grid;
    gap: 0.45rem;
  }

  .file-label {
    position: relative;
    overflow: hidden;
    display: inline-flex;
    align-items: center;
  }

  .file-label input {
    position: absolute;
    inset: 0;
    opacity: 0;
    cursor: pointer;
  }

  @media (max-width: 900px) {
    .draw-layout {
      grid-template-columns: 1fr;
    }
  }
</style>
