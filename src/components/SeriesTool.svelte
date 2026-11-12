<script lang="ts">
  import MathField from './MathField.svelte';
  import MathBlock from './Math.svelte';
  import MathText from './MathText.svelte';
  import LatexPreview from './LatexPreview.svelte';
  import Tabs from './Tabs.svelte';
  import ExportBar from './ExportBar.svelte';
  import StatusLine from './StatusLine.svelte';
  import {
    expandSeries,
    MAX_ORDER,
    type SeriesResult
  } from '../lib/series';

  type Example = 'exp' | 'sin' | 'cos' | 'ln' | 'geometric' | 'sqrt';

  let source = $state('sin(x)');
  let center = $state('0');
  let order = $state(5);
  let result = $state<SeriesResult | null>(null);
  let error = $state('');
  let tab = $state<'terms' | 'latex'>('terms');

  function run() {
    error = '';
    result = null;
    try {
      result = expandSeries(source, center, order);
      tab = 'terms';
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  function loadExample(kind: Example) {
    error = '';
    result = null;
    tab = 'terms';
    if (kind === 'exp') {
      source = 'exp(x)';
      center = '0';
      order = 5;
    } else if (kind === 'sin') {
      source = 'sin(x)';
      center = '0';
      order = 7;
    } else if (kind === 'cos') {
      source = 'cos(x)';
      center = '0';
      order = 6;
    } else if (kind === 'ln') {
      source = 'ln(1+x)';
      center = '0';
      order = 5;
    } else if (kind === 'geometric') {
      source = '1/(1-x)';
      center = '0';
      order = 5;
    } else {
      source = 'sqrt(x)';
      center = '1';
      order = 3;
    }
    run();
  }

  const seriesTitle = $derived(
    result
      ? `T_{${result.order}}(x) = ${result.seriesLatex}`
      : ''
  );

  const statusLabel = $derived(
    result
      ? result.kind === 'maclaurin'
        ? 'Maclaurin series'
        : 'Taylor series'
      : ''
  );

  run();
</script>

<form
  class="panel"
  onsubmit={(event: SubmitEvent) => {
    event.preventDefault();
    run();
  }}
>
  <h2 class="panel-title">Function</h2>

  <div class="formula-field">
    <span class="field-label">f(x)</span>
    <MathField
      bind:value={source}
      format="expression"
      variables={['x']}
      showEnter={false}
      ariaLabel="Function of x"
      onEnter={run}
    />
  </div>

  <div class="series-controls">
    <div class="formula-field">
      <span class="field-label">Centre a</span>
      <MathField
        bind:value={center}
        format="number"
        showEnter={false}
        ariaLabel="Centre of expansion"
        onEnter={run}
      />
    </div>
    <label class="field">
      <span>Degree</span>
      <input type="number" min="0" max={MAX_ORDER} bind:value={order} />
    </label>
  </div>

  <div class="example-row">
    <span class="example-label">Examples</span>
    <div class="btn-row">
      <button class="btn btn-ghost btn-small" type="button" onclick={() => loadExample('exp')}
        >e<sup>x</sup></button
      >
      <button class="btn btn-ghost btn-small" type="button" onclick={() => loadExample('sin')}
        >sin x</button
      >
      <button class="btn btn-ghost btn-small" type="button" onclick={() => loadExample('cos')}
        >cos x</button
      >
      <button class="btn btn-ghost btn-small" type="button" onclick={() => loadExample('ln')}
        >ln(1+x)</button
      >
      <button
        class="btn btn-ghost btn-small"
        type="button"
        onclick={() => loadExample('geometric')}>1/(1−x)</button
      >
      <button class="btn btn-ghost btn-small" type="button" onclick={() => loadExample('sqrt')}
        >√x at 1</button
      >
    </div>
  </div>

  <div class="btn-row">
    <button class="btn" type="submit">Expand</button>
    <button
      class="btn btn-ghost"
      type="button"
      onclick={() => {
        source = '';
        result = null;
        error = '';
      }}>Clear</button
    >
  </div>

  <p class="hint">
    Use <code>x</code>, numbers, <code>+ − * / ^</code>, parentheses, and
    <code>sin</code>, <code>cos</code>, <code>tan</code>, <code>exp</code>,
    <code>ln</code>, <code>sqrt</code>, <code>sinh</code> or <code>cosh</code>.
    Implicit multiplication is allowed: <code>2x</code>, <code>x(x+1)</code>.
  </p>

  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}
</form>

{#if result}
  <section class="panel" aria-live="polite">
    <StatusLine
      label={statusLabel}
      meta={`degree ${result.order} · ${result.order + 1} ${result.order === 0 ? 'term' : 'terms'}`}
      tone="ok"
    />

    <div class="result-math">
      <MathBlock latex={seriesTitle} display />
    </div>

    <Tabs
      ariaLabel="Series view"
      value={tab}
      items={[
        { id: 'terms', label: 'Terms' },
        { id: 'latex', label: 'preview', latex: '\\LaTeX' }
      ]}
      onChange={(id) => (tab = id as 'terms' | 'latex')}
    />

    {#if tab === 'terms'}
      <div class="series-wrap">
        <table class="series-table">
          <thead>
            <tr>
              <th scope="col"><MathText text={'$k$'} /></th>
              <th scope="col"><MathText text={'$f^{(k)}(x)$'} /></th>
              <th scope="col"><MathText text={'$f^{(k)}(a)$'} /></th>
              <th scope="col"><MathText text={'$c_k$'} /></th>
              <th scope="col"><MathText text={'$c_k(x-a)^{k}$'} /></th>
            </tr>
          </thead>
          <tbody>
            {#each result.steps as step}
              <tr>
                <td>{step.k}</td>
                <td><MathText text={`$${step.derivativeLatex}$`} /></td>
                <td><MathText text={`$${step.valueLatex}$`} /></td>
                <td><MathText text={`$${step.coefficientLatex}$`} /></td>
                <td class="series-term"><MathText text={`$${step.termLatex}$`} /></td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>

      <div class="series-notes">
        <p>
          <strong>Centre.</strong>
          {#if result.kind === 'maclaurin'}
            This is a Maclaurin series because <MathText text="$a = 0$" />.
          {:else}
            This is a Taylor series about <MathText text={`$a = ${result.centerLatex}$`} />.
          {/if}
        </p>
        <p>
          <strong>Coefficients.</strong>
          Each coefficient is <MathText text={'$c_k = f^{(k)}(a)/k!$'} />. Values at
          a non-zero centre that are not rational stay symbolic, such as
          <MathText text="$\\sin(1)$" />.
        </p>
      </div>
    {:else}
      <div class="latex-preview">
        <LatexPreview latex={result.latex} />
      </div>
      <ExportBar latex={result.latex} filename="taylor-series.tex" />
      <details class="details">
        <summary>LaTeX source</summary>
        <pre class="latex-code">{result.latex}</pre>
      </details>
    {/if}
  </section>
{/if}

<style>
  .formula-field {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    margin-bottom: 0.8rem;
  }

  .field-label {
    font-family: var(--mono);
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--fg-faint);
  }

  .series-controls {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 7rem;
    gap: 0.8rem;
    align-items: start;
  }

  .series-controls .formula-field {
    margin-bottom: 0;
  }

  .series-wrap {
    overflow-x: auto;
    margin: 0.8rem 0 0.4rem;
  }

  .series-table {
    border-collapse: collapse;
    font-family: var(--mono);
    font-size: 0.8rem;
    min-width: 100%;
  }

  .series-table th,
  .series-table td {
    border: 1px solid var(--line);
    padding: 0.4rem 0.65rem;
    text-align: center;
    white-space: nowrap;
  }

  .series-table th {
    background: var(--bg-soft);
    color: var(--fg-dim);
    font-weight: 600;
  }

  .series-table td:first-child {
    color: var(--fg-faint);
  }

  .series-table .series-term {
    color: var(--accent);
  }

  .series-notes {
    display: grid;
    gap: 0.45rem;
    margin-top: 0.9rem;
    color: var(--fg-dim);
    font-size: 0.9rem;
  }

  .series-notes p {
    margin: 0;
  }

  .series-notes strong {
    color: var(--fg);
  }

  @media (max-width: 640px) {
    .series-controls {
      grid-template-columns: 1fr;
    }

    .series-table th,
    .series-table td {
      padding: 0.35rem 0.5rem;
    }
  }
</style>
