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
  import StatusLine from './StatusLine.svelte';
  import { preloadMath } from '../lib/math';
  import { finishTableau, preload, solve, type SolveOutput } from '../lib/simplex';
  import { arDigits, localizeError } from '../lib/localize';
  import { lang } from '../lib/lang';
  import { backspaceActive, clearActive, insertActive } from '../lib/active-field';

  const VAR_NAMES: string[] = ['x', 'y', 'z', 'w', 'v'];

  type Sense = 'max' | 'min';
  type Op = '<=' | '>=' | '=';
  type Example = 'max' | 'bigm' | 'fraction' | 'infeasible';
  type InputMode = 'model' | 'tableau';
  type TableauExample = 'one-pivot' | 'bigm' | 'optimal' | 'unbounded';

  interface ConstraintForm {
    coeffs: string[];
    op: Op;
    rhs: string;
  }

  let {
    varLabels,
    senseLabels,
    senseLabelsAr,
    hintHtml,
    hintHtmlAr
  }: {
    varLabels: string[];
    senseLabels: { max: string; min: string };
    senseLabelsAr: { max: string; min: string };
    hintHtml: string;
    hintHtmlAr: string;
  } = $props();

  let ar = $state(false);
  $effect(() => {
    ar = $lang === 'ar';
  });

  let inputMode = $state<InputMode>('model');
  let method = $state<'bigm' | 'two-stage'>('bigm');
  let sense = $state<Sense>('max');
  let varCount = $state(2);
  let objective = $state<string[]>(['3', '2']);
  let constraints = $state<ConstraintForm[]>([
    { coeffs: ['1', '1'], op: '<=', rhs: '4' }
  ]);

  let busy = $state(false);
  let error = $state('');
  let result = $state<SolveOutput | null>(null);

  let step = $state(0);
  let tab = $state<'steps' | 'latex'>('steps');

  let tHeaders = $state<string[]>(['x', 'y', 's1', 's2']);
  let tRows = $state<string[][]>([
    ['1', '1', '1', '0'],
    ['1', '3', '0', '1']
  ]);
  let tRhs = $state<string[]>(['4', '6']);
  let tObj = $state<string[]>(['-3', '-2', '0', '0']);
  let tObjRhs = $state('0');
  let tBasis = $state<number[]>([2, 3]);
  let tDecision = $state<number[]>([0, 1]);
  let tArtificial = $state<number[]>([]);
  let tReportAs = $state<'max' | 'min'>('max');
  let tLabel = $state('Z');

  const vars = $derived(VAR_NAMES.slice(0, varCount));
  const current = $derived(
    result ? result.steps[Math.min(step, result.steps.length - 1)] : null
  );
  const lastIndex = $derived(result ? result.steps.length - 1 : 0);
  const statusLabel = $derived(
    result
      ? (ar
          ? { optimal: 'مثلى', unbounded: 'غير محدود', infeasible: 'غير ممكن' }
          : { optimal: 'Optimal', unbounded: 'Unbounded', infeasible: 'Infeasible' })[
          result.status
        ]
      : ''
  );

  const countsLabel = $derived(
    result
      ? ar
        ? `${arDigits(result.iterations)} ${result.iterations === 1 ? 'دورة محورية' : 'دورات محورية'} · ${arDigits(result.steps.length)} ${result.steps.length === 1 ? 'جدول' : 'جداول'}`
        : `${result.method === 'two-stage' ? 'Two-stage' : result.method === 'bigm' ? 'Big-M' : 'Tableau'} · ${result.iterations} pivot${result.iterations === 1 ? '' : 's'} · ${result.steps.length} tableau${result.steps.length === 1 ? '' : 's'}`
      : ''
  );

  function resize(next: number) {
    varCount = next;
    objective = Array.from({ length: next }, (_, i) => objective[i] ?? '0');
    constraints = constraints.map((constraint) => ({
      ...constraint,
      coeffs: Array.from(
        { length: next },
        (_, i) => constraint.coeffs[i] ?? '0'
      )
    }));
  }

  function coefficientIsNegative(value: string): boolean {
    return value.trim().startsWith('-');
  }

  function addConstraint() {
    constraints = [
      ...constraints,
      { coeffs: vars.map(() => '0'), op: '<=', rhs: '0' }
    ];
  }

  function removeConstraint(index: number) {
    constraints = constraints.filter((_, i) => i !== index);
    if (constraints.length === 0) {
      addConstraint();
    }
  }

  function toggleColumn(list: number[], column: number): number[] {
    return list.includes(column)
      ? list.filter((value) => value !== column)
      : [...list, column].sort((a, b) => a - b);
  }

  function setTableauRows(next: number) {
    if (next < 1 || next > 10) return;
    if (next < tRows.length) {
      tRows = tRows.slice(0, next);
      tRhs = tRhs.slice(0, next);
      tBasis = tBasis.slice(0, next);
    } else {
      while (tRows.length < next) {
        tRows = [...tRows, tHeaders.map(() => '0')];
        tRhs = [...tRhs, '0'];
        tBasis = [...tBasis, Math.max(0, tHeaders.length - 1)];
      }
    }
  }

  function setTableauColumns(next: number) {
    if (next < 1 || next > 16) return;
    if (next < tHeaders.length) {
      tHeaders = tHeaders.slice(0, next);
      tRows = tRows.map((row) => row.slice(0, next));
      tObj = tObj.slice(0, next);
      tBasis = tBasis.map((column) => (column < next ? column : 0));
      tDecision = tDecision.filter((column) => column < next);
      tArtificial = tArtificial.filter((column) => column < next);
    } else {
      while (tHeaders.length < next) {
        const column = tHeaders.length;
        tHeaders = [...tHeaders, `c${column + 1}`];
        tRows = tRows.map((row) => [...row, '0']);
        tObj = [...tObj, '0'];
      }
    }
  }

  function setTableauExample(kind: TableauExample) {
    inputMode = 'tableau';
    result = null;
    error = '';
    tab = 'steps';
    step = 0;
    tReportAs = 'max';
    tLabel = 'Z';
    tDecision = [0, 1];
    tArtificial = [];
    if (kind === 'one-pivot') {
      tHeaders = ['x', 'y', 's1', 's2'];
      tRows = [
        ['2/3', '0', '1', '-1/3'],
        ['1/3', '1', '0', '1/3']
      ];
      tRhs = ['2', '2'];
      tBasis = [2, 1];
      tObj = ['-1', '0', '0', '1'];
      tObjRhs = '6';
    } else if (kind === 'bigm') {
      tHeaders = ['x', 'a1'];
      tRows = [['1', '1']];
      tRhs = ['4'];
      tBasis = [1];
      tObj = ['-2', 'M'];
      tObjRhs = '0';
      tDecision = [0];
      tArtificial = [1];
    } else if (kind === 'optimal') {
      tHeaders = ['x', 'y', 's1', 's2'];
      tRows = [
        ['1', '1', '1', '0'],
        ['0', '2', '-1', '1']
      ];
      tRhs = ['4', '2'];
      tBasis = [0, 3];
      tObj = ['0', '1', '3', '0'];
      tObjRhs = '12';
    } else {
      tHeaders = ['x', 's1'];
      tRows = [['0', '1']];
      tRhs = ['1'];
      tBasis = [1];
      tObj = ['-1', '0'];
      tObjRhs = '0';
      tDecision = [0];
    }
  }

  function loadExample(kind: Example) {
    result = null;
    error = '';
    tab = 'steps';
    step = 0;
    varCount = 2;
    if (kind === 'max') {
      sense = 'max';
      objective = ['3', '2'];
      constraints = [
        { coeffs: ['1', '1'], op: '<=', rhs: '4' },
        { coeffs: ['1', '3'], op: '<=', rhs: '6' }
      ];
    } else if (kind === 'bigm') {
      sense = 'min';
      objective = ['2', '3'];
      constraints = [
        { coeffs: ['1', '1'], op: '>=', rhs: '4' },
        { coeffs: ['1', '3'], op: '>=', rhs: '6' }
      ];
    } else if (kind === 'fraction') {
      sense = 'max';
      objective = ['1/2', '1'];
      constraints = [
        { coeffs: ['1', '1'], op: '<=', rhs: '4' },
        { coeffs: ['1', '3'], op: '<=', rhs: '6' }
      ];
    } else {
      sense = 'max';
      objective = ['1', '1'];
      constraints = [
        { coeffs: ['1', '1'], op: '<=', rhs: '1' },
        { coeffs: ['1', '1'], op: '>=', rhs: '2' }
      ];
    }
  }

  async function runTableau() {
    busy = true;
    error = '';
    result = null;
    tab = 'steps';
    step = 0;
    preloadMath();
    try {
      result = await finishTableau({
        objectiveLabel: tLabel,
        reportAs: tReportAs,
        headers: tHeaders.map((header) => header.trim()),
        basis: tBasis,
        decisionColumns: tDecision,
        artificialColumns: tArtificial,
        rows: tRows,
        rhs: tRhs,
        obj: tObj,
        objRhs: tObjRhs
      });
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      error = ar ? localizeError(message) : message;
    } finally {
      busy = false;
    }
  }

  async function run() {
    if (inputMode === 'tableau') {
      await runTableau();
      return;
    }
    busy = true;
    error = '';
    result = null;
    tab = 'steps';
    step = 0;
    preloadMath();
    try {
      result = await solve({
        sense,
        method,
        vars,
        objective,
        constraints
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
    preload();
    preloadMath();
  }}
>
  <h2 class="panel-title" >{ar ? 'المسألة' : 'Problem'}</h2>

  <Tabs
    ariaLabel={ar ? 'طريقة الإدخال' : 'Input mode'}
    value={inputMode}
    items={[
      { id: 'model', label: ar ? 'من النموذج' : 'From model' },
      { id: 'tableau', label: ar ? 'أنهِ جدولاً' : 'Finish tableau' }
    ]}
    onChange={(id) => {
      inputMode = id as InputMode;
      result = null;
      error = '';
      tab = 'steps';
      step = 0;
    }}
  />

  {#if inputMode === 'model'}
  <div class="field-row">
    <label class="field">
      <span >{ar ? 'الهدف' : 'Optimise'}</span>
      <select bind:value={sense}>
        <option value="max" >{ar ? 'تعظيم' : 'Maximise'}</option>
        <option value="min" >{ar ? 'تصغير' : 'Minimise'}</option>
      </select>
    </label>

    <label class="field">
      <span >{ar ? 'المتغيرات' : 'Variables'}</span>
      <select bind:value={varCount} onchange={() => resize(varCount)}>
        <option value={2}>{ar ? arDigits(2) : 2}</option>
        <option value={3}>{ar ? arDigits(3) : 3}</option>
        <option value={4}>{ar ? arDigits(4) : 4}</option>
        <option value={5}>{ar ? arDigits(5) : 5}</option>
      </select>
    </label>

    <label class="field">
      <span>{ar ? 'الطريقة' : 'Method'}</span>
      <select bind:value={method}>
        <option value="bigm">{ar ? 'أم الكبيرة' : 'Big-M'}</option>
        <option value="two-stage">{ar ? 'المرحلتان' : 'Two-stage simplex'}</option>
      </select>
    </label>

  </div>

  <div class="example-row">
    <span class="example-label" >{ar ? 'أمثلة' : 'Examples'}</span>
    <div class="btn-row">
      <button
        class="btn btn-ghost btn-small"
        type="button"
        onclick={() => loadExample('max')} >{ar ? 'مسألة تعظيم' : 'Maximisation'}</button
      >
      <button
        class="btn btn-ghost btn-small"
        type="button"
        onclick={() => loadExample('bigm')} >{ar ? 'أم الكبيرة' : 'Big-M'}</button>
      <button
        class="btn btn-ghost btn-small"
        type="button"
        onclick={() => loadExample('fraction')} >{ar ? 'كسور' : 'Fractions'}</button>
      <button
        class="btn btn-ghost btn-small"
        type="button"
        onclick={() => loadExample('infeasible')} >{ar ? 'غير ممكن' : 'Infeasible'}</button>
    </div>
  </div>

  <div class="objective-row">
    <span class="operator">{@html (ar ? senseLabelsAr : senseLabels)[sense]}</span>
    {#each vars as variable, i}
      <span class="term">
        <MathField
          bind:value={objective[i]}
          format="number"
          keypad={false}
          compact
          showEnter={false}
          ariaLabel={`coefficient of ${variable}`}
        />
        <span class="term-var">{@html varLabels[i]}</span>
      </span>
      {#if i < vars.length - 1}
        <span class="operator"
          >{coefficientIsNegative(objective[i + 1]) ? '' : '+'}</span
        >
      {/if}
    {/each}
  </div>

  <fieldset class="constraint-list">
    <legend >{ar ? 'الخضوع لـ' : 'Subject to'}</legend>
    {#each constraints as constraint, ci}
      <div class="constraint-row">
        {#each vars as variable, i}
          <span class="term">
            <MathField
              bind:value={constraint.coeffs[i]}
              format="number"
              keypad={false}
              compact
              showEnter={false}
              ariaLabel={`constraint ${ci + 1}, coefficient of ${variable}`}
            />
            <span class="term-var">{@html varLabels[i]}</span>
          </span>
          {#if i < vars.length - 1}
            <span class="operator"
              >{coefficientIsNegative(constraint.coeffs[i + 1]) ? '' : '+'}</span
            >
          {/if}
        {/each}
        <select
          aria-label={`relation for constraint ${ci + 1}`}
          bind:value={constraint.op}
        >
          <option value="<=">≤</option>
          <option value=">=">≥</option>
          <option value="=">=</option>
        </select>
        <MathField
          bind:value={constraint.rhs}
          format="number"
          keypad={false}
          compact
          showEnter={false}
          ariaLabel={`right-hand side of constraint ${ci + 1}`}
        />
        <button
          class="btn btn-ghost btn-small"
          type="button"
          onclick={() => removeConstraint(ci)}
          aria-label={`remove constraint ${ci + 1}`} >{ar ? 'إزالة' : 'Remove'}</button
        >
      </div>
    {/each}
  </fieldset>

  <Keypad
    format="number"
    showEnter={false}
    onInsert={(latex) => insertActive(latex)}
    onBackspace={() => backspaceActive()}
    onClear={() => clearActive()}
  />

  <div class="btn-row">
    <button class="btn" type="submit" disabled={busy}>
      {ar ? (busy ? 'جارٍ الحل…' : 'حل') : (busy ? 'Solving…' : 'Solve')}
    </button>
    <button class="btn btn-ghost" type="button" onclick={addConstraint}
       >{ar ? '+ إضافة قيد' : '+ Add constraint'}</button
    >
  </div>

  {:else}
    <div class="field-row">
      <label class="field">
        <span>{ar ? 'الصفوف' : 'Rows'}</span>
        <select value={tRows.length} onchange={(event) => setTableauRows(Number(event.currentTarget.value))}>
          {#each Array(10) as _, index}
            <option value={index + 1}>{ar ? arDigits(index + 1) : index + 1}</option>
          {/each}
        </select>
      </label>
      <label class="field">
        <span>{ar ? 'الأعمدة' : 'Columns'}</span>
        <select value={tHeaders.length} onchange={(event) => setTableauColumns(Number(event.currentTarget.value))}>
          {#each Array(16) as _, index}
            <option value={index + 1}>{ar ? arDigits(index + 1) : index + 1}</option>
          {/each}
        </select>
      </label>
      <label class="field">
        <span>{ar ? 'يُبلَّغ كـ' : 'Report as'}</span>
        <select bind:value={tReportAs}>
          <option value="max">Z</option>
          <option value="min">C = −Z</option>
        </select>
      </label>
      <label class="field">
        <span>{ar ? 'اسم الهدف' : 'Objective label'}</span>
        <input class="tableau-label" bind:value={tLabel} />
      </label>
    </div>

    <div class="example-row">
      <span class="example-label">{ar ? 'أمثلة' : 'Examples'}</span>
      <div class="btn-row">
        <button class="btn btn-ghost btn-small" type="button" onclick={() => setTableauExample('one-pivot')}
          >One pivot left</button
        >
        <button class="btn btn-ghost btn-small" type="button" onclick={() => setTableauExample('bigm')}
          >Big-M</button
        >
        <button class="btn btn-ghost btn-small" type="button" onclick={() => setTableauExample('optimal')}
          >Already optimal</button
        >
        <button class="btn btn-ghost btn-small" type="button" onclick={() => setTableauExample('unbounded')}
          >Unbounded</button
        >
      </div>
    </div>

    <div class="tableau-scroll">
      <table class="tableau-grid">
        <thead>
          <tr>
            <th scope="col">b.v.</th>
            {#each tHeaders as _, column}
              <th scope="col">
                <input
                  class="tableau-header"
                  bind:value={tHeaders[column]}
                  aria-label={`column ${column + 1} name`}
                />
              </th>
            {/each}
            <th scope="col">RHS</th>
          </tr>
        </thead>
        <tbody>
          {#each tRows as _, row}
            <tr>
              <td>
                <select bind:value={tBasis[row]} aria-label={`basic variable for row ${row + 1}`}>
                  {#each tHeaders as header, column}
                    <option value={column}>{header || `c${column + 1}`}</option>
                  {/each}
                </select>
              </td>
              {#each tHeaders as _, column}
                <td>
                  <MathField
                    bind:value={tRows[row][column]}
                    format="expression"
                    variables={['M']}
                    keypad={false}
                    compact
                    showEnter={false}
                    ariaLabel={`row ${row + 1}, column ${column + 1}`}
                  />
                </td>
              {/each}
              <td>
                <MathField
                  bind:value={tRhs[row]}
                  format="expression"
                  variables={['M']}
                  keypad={false}
                  compact
                  showEnter={false}
                  ariaLabel={`right-hand side of row ${row + 1}`}
                />
              </td>
            </tr>
          {/each}
          <tr class="objective-grid-row">
            <td>{tLabel || 'Z'}</td>
            {#each tObj as _, column}
              <td>
                <MathField
                  bind:value={tObj[column]}
                  format="expression"
                  variables={['M']}
                  keypad={false}
                  compact
                  showEnter={false}
                  ariaLabel={`objective column ${column + 1}`}
                />
              </td>
            {/each}
            <td>
              <MathField
                bind:value={tObjRhs}
                format="expression"
                variables={['M']}
                keypad={false}
                compact
                showEnter={false}
                ariaLabel="objective right-hand side"
              />
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div class="tableau-picks">
      <div>
        <span>Decision columns</span>
        {#each tHeaders as header, column}
          <label>
            <input
              type="checkbox"
              checked={tDecision.includes(column)}
              onchange={() => (tDecision = toggleColumn(tDecision, column))}
            />
            {header || `c${column + 1}`}
          </label>
        {/each}
      </div>
      <div>
        <span>Artificial columns</span>
        {#each tHeaders as header, column}
          <label>
            <input
              type="checkbox"
              checked={tArtificial.includes(column)}
              onchange={() => (tArtificial = toggleColumn(tArtificial, column))}
            />
            {header || `c${column + 1}`}
          </label>
        {/each}
      </div>
    </div>

    <Keypad
      format="expression"
      variables={['M']}
      showEnter={false}
      onInsert={(latex) => insertActive(latex)}
      onBackspace={() => backspaceActive()}
      onClear={() => clearActive()}
    />

    <div class="btn-row">
      <button class="btn" type="submit" disabled={busy}>
        {ar ? (busy ? 'جارٍ الإنهاء…' : 'أنهِ الجدول') : (busy ? 'Finishing…' : 'Finish tableau')}
      </button>
      <button class="btn btn-ghost" type="button" onclick={() => setTableauRows(tRows.length + 1)}
        >+ Add row</button
      >
      <button class="btn btn-ghost" type="button" onclick={() => setTableauColumns(tHeaders.length + 1)}
        >+ Add column</button
      >
    </div>
  {/if}

  {#if inputMode === 'model'}
    <p class="hint">{@html ar ? hintHtmlAr : hintHtml}</p>
  {:else}
    <p class="hint">
      Cells accept exact rationals and Big-M expressions such as
      <code>1 + M</code>, <code>-M</code> or <code>2M</code>. Each row’s basic
      variable must be a unit column. The objective row uses the
      <code>Z − c'x + M a = 0</code> convention.
    </p>
  {/if}

  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}
</form>

{#if result}
  <section class="panel" aria-live="polite">
    <StatusLine
      label={statusLabel}
      meta={countsLabel}
      tone={result.status === 'optimal' ? 'ok' : 'warn'}
    />

    <ul class="summary">
      {#each result.summaryLatex as line}
        <li><MathText text={line} /></li>
      {/each}
    </ul>

    {#if result.standardForm}
      <details class="details">
        <summary>{ar ? 'الصورة القياسية' : 'Standard form'}</summary>
        <div class="detail-body">
          <MathBlock latex={result.standardForm.objectiveLatex} display />
          <div class="math-lines">
            {#each result.standardForm.constraintsLatex as line}
              <MathBlock latex={line} display />
            {/each}
          </div>
          <MathBlock latex={result.standardForm.objectiveRowLatex} display />
        </div>
      </details>
    {/if}

    <Tabs
      ariaLabel={ar ? 'عرض الحل' : 'Solution view'}
      value={tab}
      items={[
        { id: 'steps', label: ar ? 'الجداول' : 'Tableaus' },
        { id: 'latex', label: ar ? 'معاينة' : 'preview', latex: '\\LaTeX' }
      ]}
      onChange={(id) => (tab = id as 'steps' | 'latex')}
    />

    {#if tab === 'steps'}
      {#if current}
        <div class="tableau-wrap">
          <MathBlock
            latex={current.latex}
            display
            fallback={ar ? `الجدول ${arDigits(String(step + 1))}` : `Tableau ${step + 1}`}
          />
        </div>

        {#if current.proseLatex.length > 0}
          <ul class="step-prose">
            {#each current.proseLatex as line}
              <li><MathText text={line} /></li>
            {/each}
          </ul>
        {/if}

        <StepControls
          {step}
          last={lastIndex}
          label={ar
            ? `الجدول ${arDigits(String(step + 1))} من ${arDigits(String(result.steps.length))}`
            : `Tableau ${step + 1} of ${result.steps.length}`}
          previous={ar ? 'السابق' : '← Previous'}
          next={ar ? 'التالي' : 'Next →'}
          onStep={(next) => (step = next)}
        />
      {/if}
    {:else}
      <ExportBar
        latex={result.latex.document}
        filename="simplex-solution.tex"
        copyLabel={ar ? 'نسخ الكل' : 'Copy full'}
        downloadLabel={ar ? 'تنزيل .tex' : 'Download .tex'}
      />

      <div class="latex-preview">
        <LatexPreview latex={result.latex.document} />
      </div>

      <details class="details">
        <summary><LatexLogo /> source</summary>
        <pre class="latex-code">{result.latex.document}</pre>
      </details>
    {/if}
  </section>
{/if}


<style>
  .term :global(.mathfield),
  .constraint-row > :global(.mathfield) {
    width: 5rem;
    min-width: 3.6rem;
  }

  .tableau-scroll {
    overflow-x: auto;
    margin: 0.8rem 0;
  }

  .tableau-grid {
    border-collapse: collapse;
    font-family: var(--mono);
    font-size: 0.8rem;
  }

  .tableau-grid th,
  .tableau-grid td {
    border: 1px solid var(--line);
    padding: 0.25rem;
    text-align: center;
    white-space: nowrap;
  }

  .tableau-grid th {
    background: var(--bg-soft);
    color: var(--fg-dim);
    font-weight: 600;
  }

  .tableau-grid :global(.mathfield) {
    width: 5.5rem;
    min-width: 4rem;
  }

  .tableau-grid select {
    max-width: 5.5rem;
  }

  .tableau-header,
  .tableau-label {
    width: 4.8rem;
    padding: 0.3rem 0.4rem;
    border: 1px solid var(--line);
    border-radius: 5px;
    background: var(--bg);
    color: var(--fg);
    font-family: var(--mono);
  }

  .tableau-label {
    width: 5rem;
  }

  .objective-grid-row td {
    background: rgba(184, 166, 248, 0.08);
    color: var(--accent);
  }

  .tableau-picks {
    display: grid;
    gap: 0.55rem;
    margin: 0.8rem 0;
    font-size: 0.82rem;
  }

  .tableau-picks > div {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.55rem;
  }

  .tableau-picks > div > span {
    color: var(--fg-faint);
    font-family: var(--mono);
    font-size: 0.68rem;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .tableau-picks label {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    color: var(--fg-dim);
  }

</style>
