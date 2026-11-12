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
  import { preload, solve, type SolveOutput } from '../lib/simplex';
  import { arDigits, localizeError } from '../lib/localize';
  import { lang } from '../lib/lang';
  import { backspaceActive, clearActive, insertActive } from '../lib/active-field';

  const VAR_NAMES: string[] = ['x', 'y', 'z', 'w', 'v'];

  type Sense = 'max' | 'min';
  type Op = '<=' | '>=' | '=';
  type Example = 'max' | 'bigm' | 'fraction' | 'infeasible';

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
        : `${result.iterations} pivot${result.iterations === 1 ? '' : 's'} · ${result.steps.length} tableau${result.steps.length === 1 ? '' : 's'}`
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

  async function run() {
    busy = true;
    error = '';
    result = null;
    tab = 'steps';
    step = 0;
    preloadMath();
    try {
      result = await solve({
        sense,
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

  <p class="hint">{@html ar ? hintHtmlAr : hintHtml}</p>

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

    <details class="details">
      <summary >{ar ? 'الصورة القياسية' : 'Standard form'}</summary>
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
</style>
