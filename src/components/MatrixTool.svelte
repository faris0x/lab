<script lang="ts">
  import MathBlock from './Math.svelte';
  import MathText from './MathText.svelte';
  import LatexLogo from './LatexLogo.svelte';
  import { preloadMath } from '../lib/math';
  import {
    preloadMatrix,
    solveMatrix,
    type MatrixOperation,
    type MatrixOutput
  } from '../lib/matrix';
  import { arDigits, localizeError } from '../lib/localize';
  import { lang } from '../lib/lang';

  type Example = 'rref' | 'det' | 'inverse' | 'nullspace' | 'solve';

  let {
    labels,
    hintHtml,
    hintHtmlAr
  }: {
    labels: { A: string; augmented: string };
    hintHtml: string;
    hintHtmlAr: string;
  } = $props();

  let ar = $state(false);
  $effect(() => {
    ar = $lang === 'ar';
  });

  function blankMatrix(r: number, c: number): string[][] {
    return Array.from({ length: r }, () =>
      Array.from({ length: c }, () => '')
    );
  }

  function blankRhs(r: number): string[] {
    return Array.from({ length: r }, () => '');
  }

  let operation = $state<MatrixOperation>('rref');
  let rows = $state(3);
  let cols = $state(3);
  let matrix = $state<string[][]>(blankMatrix(3, 3));
  let rhs = $state<string[]>(blankRhs(3));

  let busy = $state(false);
  let error = $state('');
  let result = $state<MatrixOutput | null>(null);
  let step = $state(0);
  let tab = $state<'steps' | 'latex'>('steps');
  let copyState = $state('');

  const currentStep = $derived(
    result ? result.steps[Math.min(step, result.steps.length - 1)] : null
  );
  const lastIndex = $derived(result ? result.steps.length - 1 : 0);

  function resetInput() {
    rows = 3;
    cols = 3;
    matrix = blankMatrix(3, 3);
    rhs = blankRhs(3);
    result = null;
    error = '';
    step = 0;
    tab = 'steps';
    copyState = '';
  }

  function clearCells() {
    matrix = blankMatrix(rows, cols);
    rhs = blankRhs(rows);
    result = null;
    error = '';
    step = 0;
  }

  function resizeRows(next: number) {
    rows = next;
    matrix = Array.from({ length: next }, (_, i) =>
      matrix[i] ? [...matrix[i]] : Array.from({ length: cols }, () => '')
    );
    rhs = Array.from({ length: next }, (_, i) => rhs[i] ?? '');
  }

  function resizeCols(next: number) {
    cols = next;
    matrix = matrix.map((row) =>
      Array.from({ length: next }, (_, i) => row[i] ?? '')
    );
  }

  function setExample(kind: Example) {
    result = null;
    error = '';
    step = 0;
    tab = 'steps';
    copyState = '';
    operation = kind;
    if (kind === 'rref') {
      rows = 3;
      cols = 3;
      matrix = [
        ['1', '2', '1'],
        ['2', '4', '3'],
        ['3', '6', '4']
      ];
      rhs = blankRhs(3);
    } else if (kind === 'det') {
      rows = 3;
      cols = 3;
      matrix = [
        ['1', '2', '3'],
        ['0', '1', '4'],
        ['5', '6', '0']
      ];
      rhs = blankRhs(3);
    } else if (kind === 'inverse') {
      rows = 2;
      cols = 2;
      matrix = [
        ['4', '7'],
        ['2', '6']
      ];
      rhs = blankRhs(2);
    } else if (kind === 'nullspace') {
      rows = 2;
      cols = 3;
      matrix = [
        ['1', '2', '3'],
        ['2', '4', '6']
      ];
      rhs = blankRhs(2);
    } else {
      rows = 2;
      cols = 2;
      matrix = [
        ['1', '2'],
        ['3', '4']
      ];
      rhs = ['5', '6'];
    }
  }

  async function run() {
    error = '';
    if (matrix.some((row) => row.some((entry) => entry.trim() === ''))) {
      error = ar ? 'املأ كل عنصر في المصفوفة.' : 'Fill every entry in the matrix.';
      return;
    }
    if (
      operation === 'solve' &&
      rhs.some((entry) => entry.trim() === '')
    ) {
      error = ar ? 'املأ كل عنصر في الطرف الأيمن.' : 'Fill every right-hand side entry.';
      return;
    }

    busy = true;
    result = null;
    step = 0;
    tab = 'steps';
    copyState = '';
    preloadMath();
    try {
      result = await solveMatrix({
        operation,
        matrix,
        rhs: operation === 'solve' ? rhs : undefined
      });
    } catch (e) {
      error = localizeError(e instanceof Error ? e.message : String(e));
    } finally {
      busy = false;
    }
  }

  function go(delta: number) {
    step = Math.max(0, Math.min(lastIndex, step + delta));
  }

  async function copyLatex() {
    if (!result) return;
    try {
      await navigator.clipboard.writeText(result.latex.document);
      copyState = ar ? 'تم النسخ' : 'Copied';
    } catch {
      copyState = ar ? 'فشل النسخ' : 'Copy failed';
    }
    window.setTimeout(() => {
      copyState = '';
    }, 1600);
  }

  function downloadLatex() {
    if (!result) return;
    const blob = new Blob([result.latex.document], {
      type: 'application/x-tex'
    });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = `matrix-${result.operation}.tex`;
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    window.setTimeout(() => URL.revokeObjectURL(url), 1000);
  }
</script>

<form
  class="panel"
  onsubmit={(event: SubmitEvent) => {
    event.preventDefault();
    void run();
  }}
  onfocusin={() => {
    preloadMatrix();
    preloadMath();
  }}
>
  <h2 class="panel-title" >{ar ? 'المصفوفة' : 'Matrix'}</h2>

  <div class="field-row">
    <label class="field">
      <span >{ar ? 'العملية' : 'Operation'}</span>
      <select bind:value={operation} onchange={resetInput}>
        <option value="rref" >{ar ? 'اختزال بالصفوف' : 'Row reduce'}</option>
        <option value="det" >{ar ? 'المحدد' : 'Determinant'}</option>
        <option value="inverse" >{ar ? 'المعكوس' : 'Inverse'}</option>
        <option value="nullspace" >{ar ? 'الفضاء الصفري' : 'Nullspace'}</option>
        <option value="solve" >{ar ? 'حل نظام' : 'Solve a system'}</option>
      </select>
    </label>
    <label class="field">
      <span >{ar ? 'الصفوف' : 'Rows'}</span>
      <select bind:value={rows} onchange={() => resizeRows(rows)}>
        {#each [1, 2, 3, 4, 5, 6] as n}
          <option value={n}>{ar ? arDigits(n) : n}</option>
        {/each}
      </select>
    </label>
    <label class="field">
      <span >{ar ? 'الأعمدة' : 'Columns'}</span>
      <select bind:value={cols} onchange={() => resizeCols(cols)}>
        {#each [1, 2, 3, 4, 5, 6] as n}
          <option value={n}>{ar ? arDigits(n) : n}</option>
        {/each}
      </select>
    </label>
  </div>

  <div class="example-row">
    <span class="example-label" >{ar ? 'أمثلة' : 'Examples'}</span>
    <div class="btn-row">
      <button class="btn btn-ghost btn-small" type="button" onclick={() => setExample('rref')} >{ar ? 'اختزال بالصفوف' : 'Row reduce'}</button>
      <button class="btn btn-ghost btn-small" type="button" onclick={() => setExample('det')} >{ar ? 'المحدد' : 'Determinant'}</button>
      <button class="btn btn-ghost btn-small" type="button" onclick={() => setExample('inverse')} >{ar ? 'المعكوس' : 'Inverse'}</button>
      <button class="btn btn-ghost btn-small" type="button" onclick={() => setExample('nullspace')} >{ar ? 'الفضاء الصفري' : 'Nullspace'}</button>
      <button class="btn btn-ghost btn-small" type="button" onclick={() => setExample('solve')} >{ar ? 'نظام' : 'System'}</button>
    </div>
  </div>

  <div class="matrix-editor">
    <span class="matrix-label"
      >{@html operation === 'solve' ? labels.augmented : labels.A}</span
    >
    <div class="matrix-frame">
      <span class="bracket bracket-left" aria-hidden="true"></span>
      <div
        class="matrix-grid"
        style={`grid-template-columns: repeat(${cols}, 4.1rem)`}
      >
        {#each matrix as row, i}
          {#each row as entry, j}
            <input
              inputmode="decimal"
              placeholder="0"
              autocomplete="off"
              aria-label={`matrix row ${i + 1}, column ${j + 1}`}
              bind:value={matrix[i][j]}
            />
          {/each}
        {/each}
      </div>

      {#if operation === 'solve'}
        <span class="augment-bar" aria-hidden="true"></span>
        <div class="matrix-grid matrix-rhs" style="grid-template-columns: 4.1rem">
          {#each rhs as entry, i}
            <input
              inputmode="decimal"
              placeholder="0"
              autocomplete="off"
              aria-label={`right-hand side ${i + 1}`}
              bind:value={rhs[i]}
            />
          {/each}
        </div>
      {/if}

      <span class="bracket bracket-right" aria-hidden="true"></span>
    </div>
  </div>

  <div class="btn-row">
    <button class="btn" type="submit" disabled={busy}>
      {ar ? (busy ? 'جارٍ العمل…' : 'شغّل') : (busy ? 'Working…' : 'Run')}
    </button>
    <button class="btn btn-ghost" type="button" onclick={clearCells}
       >{ar ? 'مسح' : 'Clear'}</button
    >
  </div>

  <p class="hint">{@html ar ? hintHtmlAr : hintHtml}</p>

  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}
</form>

{#if result}
  <section class="panel" aria-live="polite">
    <ul class="summary">
      {#each result.summaryLatex as line}
        <li><MathText text={line} /></li>
      {/each}
    </ul>

    <div class="result-math">
      {#if result.result.matrix}
        <MathBlock
          latex={`${result.result.matrix.labelLatex} = ${result.result.matrix.latex}`}
          display
        />
      {/if}
      {#if result.result.scalar}
        <MathBlock
          latex={`${result.result.scalar.labelLatex} = ${result.result.scalar.latex}`}
          display
        />
      {/if}
      {#each result.result.vectors as vector}
        <MathBlock
          latex={`${vector.labelLatex} = ${vector.latex}`}
          display
        />
      {/each}
      {#each result.result.textLatex as line}
        <p><MathText text={line} /></p>
      {/each}
    </div>

    <details class="details">
      <summary >{ar ? 'المدخلات' : 'Input'}</summary>
      <div class="detail-body">
        <MathBlock
          latex={result.operation === 'inverse' || result.operation === 'solve'
            ? result.steps[0].matrixLatex
            : `A = ${result.inputLatex}`}
          display
        />
      </div>
    </details>

    <div class="tabs" role="tablist" aria-label={ar ? 'عرض العمل' : 'Working view'}>
      <button
        class="tab"
        role="tab"
        aria-selected={tab === 'steps'}
        onclick={() => (tab = 'steps')} >{ar ? 'الخطوات' : 'Steps'}</button
      >
      <button
        class="tab"
        role="tab"
        aria-selected={tab === 'latex'}
        onclick={() => (tab = 'latex')} >{ar ? 'معاينة ' : 'preview '}<LatexLogo /></button
      >
    </div>

    {#if tab === 'steps'}
      {#if currentStep}
        <p class="step-copy"><MathText text={currentStep.descriptionLatex} /></p>
        <div class="tableau-wrap">
          <MathBlock
            latex={currentStep.matrixLatex}
            display
            fallback={ar ? `الخطوة ${arDigits(String(step + 1))}` : `Step ${step + 1}`}
          />
        </div>

        <div class="step-controls">
          <button
            class="btn btn-ghost btn-small"
            type="button"
            disabled={step === 0}
            onclick={() => go(-1)} >{ar ? 'السابق' : '← Previous'}</button
          >
          <input
            type="range"
            min="0"
            max={lastIndex}
            bind:value={step}
            aria-label={ar ? 'خطوة المصفوفة' : 'Matrix step'}
          />
          <button
            class="btn btn-ghost btn-small"
            type="button"
            disabled={step === lastIndex}
            onclick={() => go(1)} >{ar ? 'التالي' : 'Next →'}</button
          >
          <span class="step-count"
            >{ar
              ? `الخطوة ${arDigits(String(step + 1))} من ${arDigits(String(result.steps.length))}`
              : `Step ${step + 1} of ${result.steps.length}`}</span
          >
        </div>
      {/if}
    {:else}
      <div class="btn-row" style="margin-top:0">
        <button class="btn btn-ghost btn-small" type="button" onclick={copyLatex}
          >{copyState || (ar ? 'نسخ الكل' : 'Copy full')} <LatexLogo /></button
        >
        <button class="btn btn-ghost btn-small" type="button" onclick={downloadLatex}
           >{ar ? 'تنزيل .tex' : 'Download .tex'}</button
        >
      </div>

      <div class="latex-preview">
        {#each result.latex.blocks as block}
          <MathBlock latex={block} display />
        {/each}
      </div>

      <details class="details">
        <summary><LatexLogo /> {ar ? 'المصدر' : 'source'}</summary>
        <pre class="latex-code">{result.latex.document}</pre>
      </details>
    {/if}
  </section>
{/if}
