<script lang="ts">
  import Argand from './Argand.svelte';
  import LatexLogo from './LatexLogo.svelte';
  import MathBlock from './Math.svelte';
  import MathText from './MathText.svelte';
  import { preloadMath } from '../lib/math';
  import {
    preloadComplex,
    solveComplex,
    type ComplexOperation,
    type ComplexOutput
  } from '../lib/complex';
  import { arDigits, localizeError } from '../lib/localize';
  import { lang } from '../lib/lang';

  type Example = 'multiply' | 'divide' | 'polar' | 'power' | 'roots';

  let {
    labels,
    hintHtml,
    hintHtmlAr
  }: {
    labels: { z: string; w: string; i: string };
    hintHtml: string;
    hintHtmlAr: string;
  } = $props();

  let ar = $state(false);
  $effect(() => {
    ar = $lang === 'ar';
  });

  let operation = $state<ComplexOperation>('arithmetic');
  let op = $state<'+' | '-' | '*' | '/'>('*');
  let n = $state(3);
  let zRe = $state('');
  let zIm = $state('');
  let wRe = $state('');
  let wIm = $state('');

  let busy = $state(false);
  let error = $state('');
  let result = $state<ComplexOutput | null>(null);
  let tab = $state<'steps' | 'latex'>('steps');
  let copyState = $state('');

  function negative(value: string): boolean {
    return value.trim().startsWith('-');
  }

  function clearAll() {
    zRe = '';
    zIm = '';
    wRe = '';
    wIm = '';
    result = null;
    error = '';
    tab = 'steps';
  }

  function setExample(kind: Example) {
    result = null;
    error = '';
    tab = 'steps';
    copyState = '';
    if (kind === 'multiply') {
      operation = 'arithmetic';
      op = '*';
      zRe = '3';
      zIm = '4';
      wRe = '1';
      wIm = '-2';
    } else if (kind === 'divide') {
      operation = 'arithmetic';
      op = '/';
      zRe = '1';
      zIm = '2';
      wRe = '3';
      wIm = '-4';
    } else if (kind === 'polar') {
      operation = 'polar';
      zRe = '1';
      zIm = '1';
    } else if (kind === 'power') {
      operation = 'power';
      zRe = '1';
      zIm = '1';
      n = 6;
    } else {
      operation = 'roots';
      zRe = '1';
      zIm = '0';
      n = 3;
    }
  }

  async function run() {
    error = '';
    if (zRe.trim() === '' || zIm.trim() === '') {
      error = ar ? 'املأ الجزأين الحقيقي والتخيلي لـ z.' : 'Fill both parts of z.';
      return;
    }
    if (operation === 'arithmetic' && (wRe.trim() === '' || wIm.trim() === '')) {
      error = ar ? 'املأ الجزأين الحقيقي والتخيلي لـ w.' : 'Fill both parts of w.';
      return;
    }
    busy = true;
    result = null;
    tab = 'steps';
    copyState = '';
    preloadMath();
    try {
      result = await solveComplex({
        operation,
        z: [zRe, zIm],
        w: operation === 'arithmetic' ? [wRe, wIm] : undefined,
        op: operation === 'arithmetic' ? op : undefined,
        n: operation === 'power' || operation === 'roots' ? n : undefined
      });
    } catch (e) {
      error = localizeError(e instanceof Error ? e.message : String(e));
    } finally {
      busy = false;
    }
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
    anchor.download = `complex-${result.operation}.tex`;
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
    preloadComplex();
    preloadMath();
  }}
>
  <h2 class="panel-title" >{ar ? 'الأعداد المركبة' : 'Complex numbers'}</h2>

  <div class="field-row">
    <label class="field">
      <span >{ar ? 'العملية' : 'Operation'}</span>
      <select bind:value={operation}>
        <option value="arithmetic">{ar ? 'حساب' : 'Arithmetic'}</option>
        <option value="polar">{ar ? 'الصورة القطبية' : 'Polar form'}</option>
        <option value="power" >{ar ? 'القوة' : 'Power'}</option>
        <option value="roots" >{ar ? 'الجذور' : 'Roots'}</option>
      </select>
    </label>

    {#if operation === 'arithmetic'}
      <label class="field">
        <span >{ar ? 'احسب' : 'Calculate'}</span>
        <select bind:value={op}>
          <option value="+">z + w</option>
          <option value="-">z - w</option>
          <option value="*">z × w</option>
          <option value="/">z ÷ w</option>
        </select>
      </label>
    {:else if operation === 'power' || operation === 'roots'}
      <label class="field">
        <span>{ar ? (operation === 'power' ? 'القوة' : 'دليل الجذر') : (operation === 'power' ? 'Power' : 'Root index')}</span>
        <select bind:value={n}>
          {#each operation === 'power' ? [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12] : [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12] as value}
            <option value={value}>{ar ? arDigits(value) : value}</option>
          {/each}
        </select>
      </label>
    {/if}
  </div>

  <div class="example-row">
    <span class="example-label" >{ar ? 'أمثلة' : 'Examples'}</span>
    <div class="btn-row">
      <button class="btn btn-ghost btn-small" type="button" onclick={() => setExample('multiply')} >{ar ? 'الجداء' : 'Product'}</button>
      <button class="btn btn-ghost btn-small" type="button" onclick={() => setExample('divide')} >{ar ? 'القسمة' : 'Quotient'}</button>
      <button class="btn btn-ghost btn-small" type="button" onclick={() => setExample('polar')} >{ar ? 'قطبية' : 'Polar'}</button>
      <button class="btn btn-ghost btn-small" type="button" onclick={() => setExample('power')} >{ar ? 'القوة' : 'Power'}</button>
      <button class="btn btn-ghost btn-small" type="button" onclick={() => setExample('roots')} >{ar ? 'الجذور' : 'Roots'}</button>
    </div>
  </div>

  <div class="complex-inputs">
    <div class="complex-row">
      <span class="complex-label">{@html labels.z} =</span>
      <input
        inputmode="decimal"
        placeholder="0"
        autocomplete="off"
        aria-label="real part of z"
        bind:value={zRe}
      />
      <span class="operator">{negative(zIm) ? '' : '+'}</span>
      <input
        inputmode="decimal"
        placeholder="0"
        autocomplete="off"
        aria-label="imaginary part of z"
        bind:value={zIm}
      />
      <span class="complex-i">{@html labels.i}</span>
    </div>

    {#if operation === 'arithmetic'}
      <div class="complex-row">
        <span class="complex-label">{@html labels.w} =</span>
        <input
          inputmode="decimal"
          placeholder="0"
          autocomplete="off"
          aria-label="real part of w"
          bind:value={wRe}
        />
        <span class="operator">{negative(wIm) ? '' : '+'}</span>
        <input
          inputmode="decimal"
          placeholder="0"
          autocomplete="off"
          aria-label="imaginary part of w"
          bind:value={wIm}
        />
        <span class="complex-i">{@html labels.i}</span>
      </div>
    {/if}
  </div>

  <div class="btn-row">
    <button class="btn" type="submit" disabled={busy}>
      {ar ? (busy ? 'جارٍ العمل…' : 'شغّل') : (busy ? 'Working…' : 'Run')}
    </button>
    <button class="btn btn-ghost" type="button" onclick={clearAll} >{ar ? 'مسح' : 'Clear'}</button>
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
      {#each result.lines as line}
        <MathBlock latex={line} display />
      {/each}
    </div>

    <Argand points={result.argand.points} circles={result.argand.circles} />

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
      <ul class="step-prose">
        {#each result.steps as step}
          <li><MathText text={step.descriptionLatex} /></li>
        {/each}
      </ul>
    {:else}
      <div class="btn-row" style="margin-top:0">
        <button class="btn btn-ghost btn-small" type="button" onclick={copyLatex}
          >{copyState || (ar ? 'نسخ الكل' : 'Copy full')} <LatexLogo /></button
        >
        <button class="btn btn-ghost btn-small" type="button" onclick={downloadLatex}
           >{ar ? 'تنزيل .tex' : 'Download .tex'}</button>
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
