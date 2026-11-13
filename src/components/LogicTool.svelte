<script lang="ts">
  import MathField from './MathField.svelte';
  import MathBlock from './Math.svelte';
  import LatexPreview from './LatexPreview.svelte';
  import MathText from './MathText.svelte';
  import Tabs from './Tabs.svelte';
  import ExportBar from './ExportBar.svelte';
  import { build, preloadLogic, type LogicResult } from '../lib/logic';
  import { logicLabelToLatex } from '../lib/math-input';

  type Example = 'modus' | 'demorgan' | 'tautology' | 'contradiction';

  let formula = $state('((p -> q) & p) -> q');
  let result = $state<LogicResult | null>(null);
  let error = $state('');
  let tab = $state<'table' | 'latex'>('table');

  async function run() {
    error = '';
    result = null;
    try {
      result = await build(formula);
      tab = 'table';
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  function loadExample(kind: Example) {
    error = '';
    result = null;
    tab = 'table';
    if (kind === 'modus') formula = '((p -> q) & p) -> q';
    else if (kind === 'demorgan') formula = '~(p & q) <-> (~p | ~q)';
    else if (kind === 'tautology') formula = 'p | ~p';
    else formula = 'p & ~p';
    void run();
  }

  const verdictLabel = $derived(
    result
      ? { tautology: 'Tautology', contradiction: 'Contradiction', contingent: 'Contingent' }[
          result.verdict
        ]
      : ''
  );

  preloadLogic();
  void run();
</script>

<form
  class="panel"
  onsubmit={(event: SubmitEvent) => {
    event.preventDefault();
    void run();
  }}
>
  <h2 class="panel-title">Formula</h2>

  <div class="formula-field">
    <span class="field-label">Proposition</span>
    <MathField bind:value={formula} format="logic" showEnter={false} ariaLabel="Propositional formula" onEnter={() => void run()} />
  </div>

  <div class="example-row">
    <span class="example-label">Examples</span>
    <div class="btn-row">
      <button class="btn btn-ghost btn-small" type="button" onclick={() => loadExample('modus')}
        >Modus ponens</button
      >
      <button class="btn btn-ghost btn-small" type="button" onclick={() => loadExample('demorgan')}
        >De Morgan</button
      >
      <button class="btn btn-ghost btn-small" type="button" onclick={() => loadExample('tautology')}
        >Tautology</button
      >
      <button
        class="btn btn-ghost btn-small"
        type="button"
        onclick={() => loadExample('contradiction')}>Contradiction</button
      >
    </div>
  </div>

  <div class="btn-row">
    <button class="btn" type="submit">Build table</button>
    <button class="btn btn-ghost" type="button" onclick={() => (formula = '')}>Clear</button>
  </div>

  <p class="hint">
    Use the keypad for logical symbols, or type letters and the operators
    <code>~</code>, <code>&amp;</code>, <code>|</code>, <code>^</code>,
    <code>-&gt;</code> and <code>&lt;-&gt;</code>.
  </p>

  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}
</form>

{#if result}
  <section class="panel" aria-live="polite">
    <div class="status-line">
      <span
        class="status"
        class:status-ok={result.verdict === 'tautology'}
        class:status-warn={result.verdict === 'contradiction'}>{verdictLabel}</span
      >
      <span class="step-count"
        >{result.rows.length} row{result.rows.length === 1 ? '' : 's'} ·
        {result.variables.length} variable{result.variables.length === 1 ? '' : 's'}</span
      >
    </div>

    <div class="result-math">
      <MathBlock latex={logicLabelToLatex(result.formula)} display />
    </div>

    <Tabs
      ariaLabel="Truth table view"
      value={tab}
      items={[
        { id: 'table', label: 'Table' },
        { id: 'latex', label: 'preview', latex: '\\LaTeX' }
      ]}
      onChange={(id) => (tab = id as 'table' | 'latex')}
    />

    {#if tab === 'table'}
      <div class="truth-wrap">
        <table class="truth-table">
          <thead>
            <tr>
              {#each result.columns as column}
                <th scope="col"><MathText text={`$${logicLabelToLatex(column.label)}$`} /></th>
              {/each}
            </tr>
          </thead>
          <tbody>
            {#each result.rows as row}
              <tr>
                {#each row.values as value}
                  <td class:truth-true={value} class:truth-false={!value}
                    ><MathText text={value ? '$\\top$' : '$\\bot$'} /></td
                  >
                {/each}
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
      <p class="truth-verdict">
        {#if result.verdict === 'tautology'}
          Every row is true, so the formula is a tautology.
        {:else if result.verdict === 'contradiction'}
          Every row is false, so the formula is a contradiction.
        {:else}
          The formula is true in some rows and false in others, so it is contingent.
        {/if}
      </p>
    {:else}
      <div class="latex-preview">
        <LatexPreview latex={result.latex} />
      </div>
      <ExportBar latex={result.latex} filename="truth-table.tex" />
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

  .truth-wrap {
    overflow-x: auto;
    margin: 0.8rem 0 0.4rem;
  }

  .truth-table {
    border-collapse: collapse;
    font-family: var(--mono);
    font-size: 0.82rem;
    min-width: 100%;
  }

  .truth-table th,
  .truth-table td {
    border: 1px solid var(--line);
    padding: 0.4rem 0.7rem;
    text-align: center;
    white-space: nowrap;
  }

  .truth-table th {
    background: var(--bg-soft);
    color: var(--fg-dim);
    font-weight: 600;
  }

  .truth-table th:last-child,
  .truth-table td:last-child {
    background: rgba(184, 166, 248, 0.08);
    color: var(--accent);
  }

  .truth-true {
    color: var(--ok);
  }

  .truth-false {
    color: var(--danger);
  }

  .truth-verdict {
    margin: 0.7rem 0 0;
    color: var(--fg-dim);
    font-size: 0.92rem;
  }

  @media (max-width: 640px) {
    .truth-table th,
    .truth-table td {
      padding: 0.35rem 0.5rem;
    }
  }
</style>
