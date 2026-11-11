<script lang="ts">
  import LatexLogo from './LatexLogo.svelte';
  import { build, type LogicResult } from '../lib/logic';

  type Example = 'modus' | 'demorgan' | 'tautology' | 'contradiction';

  let formula = $state('((p -> q) & p) -> q');
  let result = $state<LogicResult | null>(null);
  let error = $state('');
  let tab = $state<'table' | 'latex'>('table');
  let copyState = $state('');

  function run() {
    error = '';
    copyState = '';
    result = null;
    try {
      result = build(formula);
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
    run();
  }

  async function copyLatex() {
    if (!result) return;
    try {
      await navigator.clipboard.writeText(result.latex);
      copyState = 'Copied';
    } catch {
      copyState = 'Copy failed';
    }
    window.setTimeout(() => {
      copyState = '';
    }, 1600);
  }

  function downloadLatex() {
    if (!result) return;
    const blob = new Blob([result.latex], { type: 'application/x-tex' });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = 'truth-table.tex';
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    window.setTimeout(() => URL.revokeObjectURL(url), 1000);
  }

  const verdictLabel = $derived(
    result
      ? { tautology: 'Tautology', contradiction: 'Contradiction', contingent: 'Contingent' }[
          result.verdict
        ]
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
  <h2 class="panel-title">Formula</h2>

  <label class="field formula-field">
    <span>Proposition</span>
    <input
      bind:value={formula}
      aria-label="Propositional formula"
      spellcheck="false"
      autocomplete="off"
    />
  </label>

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
    Use letters for propositions, <code>~</code> for not, <code>&amp;</code> for and,
    <code>|</code> for or, <code>-&gt;</code> for implication and <code>&lt;-&gt;</code>
    for equivalence. Parentheses work as expected.
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

    <div class="tabs" role="tablist" aria-label="Truth table view">
      <button
        class="tab"
        role="tab"
        aria-selected={tab === 'table'}
        onclick={() => (tab = 'table')}>Table</button
      >
      <button
        class="tab"
        role="tab"
        aria-selected={tab === 'latex'}
        onclick={() => (tab = 'latex')}><LatexLogo /> preview</button
      >
    </div>

    {#if tab === 'table'}
      <div class="truth-wrap">
        <table class="truth-table">
          <thead>
            <tr>
              {#each result.columns as column}
                <th scope="col">{column.label}</th>
              {/each}
            </tr>
          </thead>
          <tbody>
            {#each result.rows as row}
              <tr>
                {#each row.values as value}
                  <td class:truth-true={value} class:truth-false={!value}>{value ? 'T' : 'F'}</td>
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
      <div class="btn-row" style="margin-top:0">
        <button class="btn btn-ghost btn-small" type="button" onclick={copyLatex}
          >{copyState || 'Copy full'} <LatexLogo /></button
        >
        <button class="btn btn-ghost btn-small" type="button" onclick={downloadLatex}
          >Download .tex</button
        >
      </div>
      <pre class="latex-code">{result.latex}</pre>
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

  .formula-field input {
    width: 100%;
    font-family: var(--mono);
    font-size: 0.95rem;
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
