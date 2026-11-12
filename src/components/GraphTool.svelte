<script lang="ts">
  import MathBlock from './Math.svelte';
  import LatexPreview from './LatexPreview.svelte';
  import MathText from './MathText.svelte';
  import MathField from './MathField.svelte';
  import Keypad from './Keypad.svelte';
  import Tabs from './Tabs.svelte';
  import ExportBar from './ExportBar.svelte';
  import StatusLine from './StatusLine.svelte';
  import {
    preloadGraph,
    runGraph,
    type GraphAlgorithm,
    type GraphResult
  } from '../lib/graph';
  import { backspaceActive, clearActive, insertActive } from '../lib/active-field';

  const initialLabels = ['A', 'B', 'C', 'D', 'E'];
  const initialWeights: string[][] = Array.from({ length: 5 }, () => Array(5).fill(''));
  initialWeights[0][1] = '4';
  initialWeights[0][2] = '2';
  initialWeights[1][2] = '1';
  initialWeights[1][3] = '5';
  initialWeights[2][3] = '8';
  initialWeights[2][4] = '10';
  initialWeights[3][4] = '2';
  for (let i = 0; i < 5; i += 1) {
    for (let j = i + 1; j < 5; j += 1) initialWeights[j][i] = initialWeights[i][j];
  }

  let labelText = $state('A, B, C, D, E');
  let labels = $state([...initialLabels]);
  let weights = $state(initialWeights.map((row) => [...row]));
  let directed = $state(false);
  let algorithm = $state<GraphAlgorithm>('dijkstra');
  let source = $state(0);
  let target = $state(4);
  let result = $state<GraphResult | null>(null);
  let error = $state('');
  let busy = $state(false);
  let tab = $state<'steps' | 'latex'>('steps');

  function applyLabels() {
    const next = labelText
      .split(',')
      .map((label) => label.trim())
      .filter(Boolean);
    if (next.length < 2 || next.length > 9) {
      error = 'Use between 2 and 9 node labels.';
      return;
    }
    if (next.some((label) => !/^[A-Za-z0-9]$/.test(label))) {
      error = 'Use one letter or digit per node.';
      return;
    }
    labels = next.map((label) => label.toUpperCase());
    weights = Array.from({ length: labels.length }, (_, i) =>
      Array.from({ length: labels.length }, (_, j) => weights[i]?.[j] ?? '')
    );
    source = Math.min(source, labels.length - 1);
    target = Math.min(target, labels.length - 1);
    error = '';
    result = null;
  }

  function setWeight(i: number, j: number, value: string) {
    const next = weights.map((row) => [...row]);
    next[i][j] = value;
    if (!directed) next[j][i] = value;
    weights = next;
  }

  async function run() {
    error = '';
    result = null;
    busy = true;
    preloadGraph();
    try {
      result = await runGraph({ labels, directed, weights }, algorithm, source, target);
      tab = 'steps';
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  run();
</script>

<form
  class="panel"
  onsubmit={(event: SubmitEvent) => {
    event.preventDefault();
    void run();
  }}
  onfocusin={() => preloadGraph()}
>
  <h2 class="panel-title">Graph</h2>

  <div class="graph-controls">
    <label class="field">
      <span>Node labels</span>
      <input bind:value={labelText} onblur={applyLabels} aria-label="Node labels" />
    </label>
    <label class="field">
      <span>Algorithm</span>
      <select bind:value={algorithm}>
        <option value="dijkstra">Dijkstra</option>
        <option value="floyd">Floyd–Warshall</option>
        <option value="kruskal">Kruskal</option>
        <option value="prim">Prim</option>
      </select>
    </label>
    <label class="field">
      <span>Start / source</span>
      <select bind:value={source}>
        {#each labels as label, index}<option value={index}>{label}</option>{/each}
      </select>
    </label>
    {#if algorithm === 'dijkstra'}
      <label class="field">
        <span>Target</span>
        <select bind:value={target}>
          {#each labels as label, index}<option value={index}>{label}</option>{/each}
        </select>
      </label>
    {/if}
    <label class="graph-check">
      <input type="checkbox" bind:checked={directed} />
      <span>Directed</span>
    </label>
  </div>

  <p class="example-label">Weighted adjacency matrix</p>
  <div class="graph-matrix-wrap">
    <table class="graph-matrix">
      <thead>
        <tr>
          <th></th>
          {#each labels as label}<th><MathText text={`$${label}$`} /></th>{/each}
        </tr>
      </thead>
      <tbody>
        {#each labels as rowLabel, i}
          <tr>
            <th><MathText text={`$${rowLabel}$`} /></th>
            {#each labels as _column, j}
              <td>
                <MathField
                  bind:value={weights[i][j]}
                  format="number"
                  keypad={false}
                  compact
                  showEnter={false}
                  ariaLabel={`${rowLabel} to ${labels[j]} weight`}
                  onValueChange={(value) => setWeight(i, j, value)}
                />
              </td>
            {/each}
          </tr>
        {/each}
      </tbody>
    </table>
  </div>

  <Keypad
    format="number"
    showEnter={false}
    onInsert={(latex) => insertActive(latex)}
    onBackspace={() => backspaceActive()}
    onClear={() => clearActive()}
  />

  <div class="btn-row">
    <button class="btn" type="submit" disabled={busy}>{busy ? 'Running…' : 'Run algorithm'}</button>
    <button class="btn btn-ghost" type="button" onclick={applyLabels}>Apply labels</button>
  </div>

  <p class="hint">
    Leave a cell blank for no edge. Weights may be integers, decimals or
    fractions; the Rust/WASM engine keeps them exact. The matrix is mirrored
    automatically for undirected graphs. Spanning trees need an undirected graph.
  </p>

  {#if error}<p class="error" role="alert">{error}</p>{/if}
</form>

{#if result}
  <section class="panel" aria-live="polite">
    <StatusLine
      label={result.title}
      meta={`${result.steps.length} step${result.steps.length === 1 ? '' : 's'}`}
      tone="ok"
    />

    <ul class="summary">
      {#each result.summary as line}<li><MathText text={line} /></li>{/each}
    </ul>

    <Tabs
      ariaLabel="Graph view"
      value={tab}
      items={[
        { id: 'steps', label: 'Working' },
        { id: 'latex', label: 'preview', latex: '\\LaTeX' }
      ]}
      onChange={(id) => (tab = id as 'steps' | 'latex')}
    />

    {#if tab === 'steps'}
      <ol class="graph-steps">
        {#each result.steps as step}
          <li>
            <MathText text={step.text} />
            {#if step.latex}<MathBlock latex={step.latex} display />{/if}
          </li>
        {/each}
      </ol>
    {:else}
      <div class="latex-preview">
        <LatexPreview latex={result.latex} />
      </div>
      <ExportBar latex={result.latex} filename={`graph-${algorithm}.tex`} />
      <details class="details">
        <summary>LaTeX source</summary>
        <pre class="latex-code">{result.latex}</pre>
      </details>
    {/if}
  </section>
{/if}

<style>
  .graph-controls {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(9rem, 1fr));
    gap: 0.7rem;
    align-items: end;
    margin-bottom: 0.9rem;
  }

  .graph-check {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    padding-bottom: 0.45rem;
    color: var(--fg-dim);
    font-size: 0.85rem;
  }

  .graph-check input {
    width: auto;
  }

  .graph-matrix-wrap {
    overflow-x: auto;
    padding-bottom: 0.25rem;
  }

  .graph-matrix {
    border-collapse: collapse;
    font-family: var(--mono);
  }

  .graph-matrix th,
  .graph-matrix td {
    border: 1px solid var(--line);
    padding: 0.25rem;
    text-align: center;
    min-width: 3.2rem;
  }

  .graph-matrix th {
    color: var(--fg-faint);
    font-size: 0.72rem;
    background: var(--bg-soft);
  }

  .graph-matrix :global(.mathfield) {
    width: 100%;
    min-width: 2.6rem;
  }

  .graph-steps {
    display: grid;
    gap: 0.85rem;
    margin-top: 0.8rem;
  }

  .graph-steps > li {
    padding-left: 0.2rem;
    color: var(--fg-dim);
    font-size: 0.92rem;
  }

  .graph-steps > li::marker {
    color: var(--accent);
    font-family: var(--mono);
  }
</style>
