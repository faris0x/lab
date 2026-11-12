<script lang="ts">
  import MathBlock from './Math.svelte';
  import LatexPreview from './LatexPreview.svelte';
  import MathText from './MathText.svelte';
  import MathField from './MathField.svelte';
  import Keypad from './Keypad.svelte';
  import LatexLogo from './LatexLogo.svelte';
  import Tabs from './Tabs.svelte';
  import ExportBar from './ExportBar.svelte';
  import StepControls from './StepControls.svelte';
  import { preloadMath } from '../lib/math';
  import {
    preloadMatrix,
    solveMatrix,
    type MatrixOperation,
    type MatrixOutput
  } from '../lib/matrix';
  import { arDigits, localizeError } from '../lib/localize';
  import { lang } from '../lib/lang';
  import { backspaceActive, clearActive, insertActive } from '../lib/active-field';

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
    preloadMath();
    try {
      result = await solveMatrix({
        operation,
        matrix,
        rhs: operation === 'solve' ? rhs : undefined
      });
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      error = ar ? localizeError(message) : message;
    } finally {
      busy = false;
    }
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
            <MathField
              bind:value={matrix[i][j]}
              format="number"
              keypad={false}
              compact
              ariaLabel={`matrix row ${i + 1}, column ${j + 1}`}
            />
          {/each}
        {/each}
      </div>

      {#if operation === 'solve'}
        <span class="augment-bar" aria-hidden="true"></span>
        <div class="matrix-grid matrix-rhs" style="grid-template-columns: 4.1rem">
          {#each rhs as entry, i}
            <MathField
              bind:value={rhs[i]}
              format="number"
              keypad={false}
              compact
              ariaLabel={`right-hand side ${i + 1}`}
            />
          {/each}
        </div>
      {/if}

      <span class="bracket bracket-right" aria-hidden="true"></span>
    </div>
  </div>

  <Keypad
    format="number"
    showEnter={false}
    onInsert={(latex) => insertActive(latex)}
    onBackspace={() => backspaceActive()}
    onClear={() => clearActive()}
    onEnter={() => void run()}
  />

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

    <Tabs
      ariaLabel={ar ? 'عرض العمل' : 'Working view'}
      value={tab}
      items={[
        { id: 'steps', label: ar ? 'الخطوات' : 'Steps' },
        { id: 'latex', label: ar ? 'معاينة' : 'preview', latex: '\\LaTeX' }
      ]}
      onChange={(id) => (tab = id as 'steps' | 'latex')}
    />

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

        <StepControls
          {step}
          last={lastIndex}
          label={ar
            ? `الخطوة ${arDigits(String(step + 1))} من ${arDigits(String(result.steps.length))}`
            : `Step ${step + 1} of ${result.steps.length}`}
          previous={ar ? 'السابق' : '← Previous'}
          next={ar ? 'التالي' : 'Next →'}
          onStep={(next) => (step = next)}
        />
      {/if}
    {:else}
      <ExportBar
        latex={result.latex.document}
        filename={`matrix-${result.operation}.tex`}
        copyLabel={ar ? 'نسخ الكل' : 'Copy full'}
        downloadLabel={ar ? 'تنزيل .tex' : 'Download .tex'}
      />

      <div class="latex-preview">
        <LatexPreview latex={result.latex.document} />
      </div>

      <details class="details">
        <summary><LatexLogo /> {ar ? 'المصدر' : 'source'}</summary>
        <pre class="latex-code">{result.latex.document}</pre>
      </details>
    {/if}
  </section>
{/if}


<style>
  .matrix-grid :global(.mathfield) {
    width: 100%;
    min-width: 0;
  }

  .matrix-rhs :global(.mathfield) {
    width: 100%;
  }
</style>
