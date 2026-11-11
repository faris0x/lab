<script lang="ts">
  import LatexLogo from './LatexLogo.svelte';
  import { simplifyBoolean, type BooleanResult } from '../lib/boolean';

  let formula = $state('(p & q) | (p & ~q)');
  let result = $state<BooleanResult | null>(null);
  let error = $state('');
  let tab = $state<'table' | 'simplify' | 'latex'>('simplify');
  let copyState = $state('');

  function run() {
    error = '';
    copyState = '';
    result = null;
    try {
      result = simplifyBoolean(formula);
      tab = 'simplify';
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  function example(source: string) {
    formula = source;
    error = '';
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
    anchor.download = 'boolean-simplification.tex';
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    window.setTimeout(() => URL.revokeObjectURL(url), 1000);
  }

  run();
</script>

<form
  class="panel"
  onsubmit={(event: SubmitEvent) => {
    event.preventDefault();
    run();
  }}
>
  <h2 class="panel-title">Boolean expression</h2>

  <label class="field formula-field">
    <span>Statement</span>
    <input
      bind:value={formula}
      aria-label="Boolean expression"
      spellcheck="false"
      autocomplete="off"
    />
  </label>

  <div class="example-row">
    <span class="example-label">Examples</span>
    <div class="btn-row">
      <button class="btn btn-ghost btn-small" type="button" onclick={() => example('(p & q) | (p & ~q)')}
        >Absorption</button
      >
      <button class="btn btn-ghost btn-small" type="button" onclick={() => example('~(p & q) <-> (~p | ~q)')}
        >De Morgan</button
      >
      <button class="btn btn-ghost btn-small" type="button" onclick={() => example('(p & q) | (~p & r) | (q & r)')}
        >Consensus</button
      >
      <button class="btn btn-ghost btn-small" type="button" onclick={() => example('p ^ q ^ r')}
        >XOR</button
      >
    </div>
  </div>

  <div class="btn-row">
    <button class="btn" type="submit">Simplify</button>
    <button class="btn btn-ghost" type="button" onclick={() => (formula = '')}>Clear</button>
  </div>

  <p class="hint">
    Use <code>~</code> for not, <code>&amp;</code> for and, <code>|</code> for or,
    <code>^</code> for xor, <code>-&gt;</code> for implication, <code>&lt;-&gt;</code>
    for equivalence, and <code>0</code>/<code>1</code> for constants.
  </p>

  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}
</form>

{#if result}
  <section class="panel" aria-live="polite">
    <div class="status-line">
      <span class="status status-ok">{result.minimal}</span>
      <span class="step-count"
        >{result.minterms.length} minterm{result.minterms.length === 1 ? '' : 's'} ·
        {result.selected.length} term{result.selected.length === 1 ? '' : 's'} after minimising</span
      >
    </div>

    <div class="tabs" role="tablist" aria-label="Boolean view">
      <button class="tab" role="tab" aria-selected={tab === 'simplify'} onclick={() => (tab = 'simplify')}
        >Simplification</button
      >
      <button class="tab" role="tab" aria-selected={tab === 'table'} onclick={() => (tab = 'table')}
        >Truth table</button
      >
      <button class="tab" role="tab" aria-selected={tab === 'latex'} onclick={() => (tab = 'latex')}
        ><LatexLogo /> preview</button
      >
    </div>

    {#if tab === 'simplify'}
      <div class="bool-grid">
        <div class="bool-card">
          <h3>Minterms and maxterms</h3>
          <p><strong>Σm</strong> = {result.minterms.length ? result.minterms.join(', ') : '—'}</p>
          <p><strong>ΠM</strong> = {result.maxterms.length ? result.maxterms.join(', ') : '—'}</p>
        </div>
        <div class="bool-card">
          <h3>Minimal sum of products</h3>
          <p class="bool-expression">{result.minimal}</p>
        </div>
        <div class="bool-card">
          <h3>Canonical SOP</h3>
          <p class="bool-expression bool-small">{result.canonicalSop}</p>
        </div>
        <div class="bool-card">
          <h3>Canonical POS</h3>
          <p class="bool-expression bool-small">{result.canonicalPos}</p>
        </div>
      </div>

      <h3 class="bool-subtitle">Prime implicants</h3>
      <ul class="prime-list">
        {#each result.primes as prime}
          <li>
            <code>{prime.expression}</code>
            <span>covers {prime.covers.join(', ')}</span>
            {#if result.essential.includes(prime)}<b>essential</b>{/if}
            {#if result.selected.includes(prime) && !result.essential.includes(prime)}<b>selected</b>{/if}
          </li>
        {/each}
      </ul>
    {:else if tab === 'table'}
      <div class="truth-wrap">
        <table class="truth-table">
          <thead>
            <tr>
              {#each result.variables as variable}<th>{variable}</th>{/each}
              <th>F</th>
            </tr>
          </thead>
          <tbody>
            {#each result.rows as row}
              <tr>
                {#each row.values as value}
                  <td class:truth-true={value} class:truth-false={!value}>{value ? '1' : '0'}</td>
                {/each}
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
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

  .bool-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(15rem, 1fr));
    gap: 0.7rem;
    margin: 0.9rem 0;
  }

  .bool-card {
    border: 1px solid var(--line);
    border-radius: var(--radius);
    padding: 0.85rem 1rem;
    background: var(--bg-soft);
  }

  .bool-card h3 {
    font-size: 0.72rem;
    font-family: var(--mono);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--fg-faint);
    margin: 0 0 0.45rem;
  }

  .bool-card p {
    margin: 0.15rem 0;
    font-size: 0.9rem;
  }

  .bool-expression {
    color: var(--accent);
    font-family: var(--mono);
    font-size: 0.92rem;
    line-height: 1.6;
    overflow-wrap: anywhere;
  }

  .bool-small {
    font-size: 0.8rem;
    color: var(--fg-dim);
  }

  .bool-subtitle {
    font-size: 1rem;
    margin: 1rem 0 0.5rem;
  }

  .prime-list {
    display: grid;
    gap: 0.35rem;
    font-size: 0.85rem;
  }

  .prime-list li {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 0.6rem;
    padding: 0.4rem 0.6rem;
    border: 1px solid var(--line-soft);
    border-radius: 7px;
  }

  .prime-list code {
    color: var(--accent);
  }

  .prime-list span {
    color: var(--fg-faint);
  }

  .prime-list b {
    font-size: 0.68rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--fg-dim);
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

  .truth-true {
    color: var(--ok);
  }

  .truth-false {
    color: var(--danger);
  }
</style>
