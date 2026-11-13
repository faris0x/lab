<script lang="ts">
  import MathField from './MathField.svelte';
  import MathLabel from './MathLabel.svelte';
  import MathBlock from './Math.svelte';
  import LatexPreview from './LatexPreview.svelte';
  import MathText from './MathText.svelte';
  import Tabs from './Tabs.svelte';
  import ExportBar from './ExportBar.svelte';
  import StatusLine from './StatusLine.svelte';
  import { escapeLatex, latexDocument } from '../lib/latex';
  import {
    convertBases,
    decodeCustom,
    decodeIeee,
    encodeCustom,
    encodeIeee,
    normalise,
    preloadFloating,
    type Base,
    type BaseResult,
    type CustomFloatResult,
    type IeeeResult,
    type NormalisedResult
  } from '../lib/floating';

  type Mode = 'convert' | 'normalise' | 'custom' | 'ieee';

  let mode = $state<Mode>('convert');
  let input = $state('1011.101');
  let from = $state<Base>(2);
  let exponentBits = $state(4);
  let mantissaBits = $state(8);
  let customAction = $state<'encode' | 'decode'>('encode');
  let ieeeKind = $state<'single' | 'double'>('single');
  let ieeeAction = $state<'encode' | 'decode'>('encode');
  let error = $state('');

  let baseResult = $state<BaseResult | null>(null);
  let normalResult = $state<NormalisedResult | null>(null);
  let customResult = $state<CustomFloatResult | null>(null);
  let ieeeResult = $state<IeeeResult | null>(null);

  const inputFormat = $derived(
    mode === 'convert' || mode === 'normalise'
      ? ('base' as const)
      : mode === 'custom'
        ? customAction === 'encode'
          ? ('number' as const)
          : ('base' as const)
        : ieeeAction === 'encode'
          ? ('number' as const)
          : ('base' as const)
  );

  const inputBase = $derived(
    mode === 'convert' || mode === 'normalise'
      ? from
      : mode === 'custom' && customAction === 'decode'
        ? (16 as Base)
        : mode === 'ieee' && ieeeAction === 'decode'
          ? (16 as Base)
          : (10 as Base)
  );

  const inputLabel = $derived(
    mode === 'custom' && customAction === 'decode'
      ? 'Bit pattern'
      : mode === 'ieee' && ieeeAction === 'decode'
        ? 'Hexadecimal bits'
        : 'Value'
  );

  function clearResults() {
    baseResult = null;
    normalResult = null;
    customResult = null;
    ieeeResult = null;
    error = '';
  }

  function bitFields(bits: string, exponentWidth: number): string {
    const sign = bits.slice(0, 1);
    const exponent = bits.slice(1, 1 + exponentWidth);
    const fraction = bits.slice(1 + exponentWidth);
    return `\\underbrace{\\texttt{${sign}}}_{\\text{sign}}\\;\\underbrace{\\texttt{${exponent}}}_{\\text{exponent}}\\;\\underbrace{\\texttt{${fraction}}}_{\\text{fraction}}`;
  }

  function exportLatex(): string {
    if (!baseResult && !normalResult && !customResult && !ieeeResult) return '';
    const sections: { title: string; body: string }[] = [];

    if (baseResult) {
      const rows = baseResult.outputs
        .map((output) => `\\text{Base ${output.base}} & \\texttt{${escapeLatex(output.text)}} & ${output.exact ? '\\text{exact}' : '\\text{recurring}'}`)
        .join(' \\\\\n');
      sections.push({
        title: 'Base conversion',
        body: `\\[\n\\begin{array}{lll}\n${rows}\n\\end{array}\n\\]`
      });
    }

    if (normalResult) {
      const sign = normalResult.sign === '-' ? '-' : '';
      const fraction = normalResult.fraction || '0';
      sections.push({
        title: 'Normalised binary',
        body: `\\[\n${sign}1.${fraction} \\times 2^{${normalResult.exponent}}\n\\]`
      });
    }

    if (customResult) {
      sections.push({
        title: 'Custom binary format',
        body: `\\[\n${bitFields(customResult.bitString, exponentBits)}\n\\]\n\\[\n\\text{Value} = ${escapeLatex(customResult.value)}\n\\]`
      });
    }

    if (ieeeResult) {
      const exponentWidth = ieeeResult.kind === 'single' ? 8 : 11;
      sections.push({
        title: `IEEE 754 ${ieeeResult.kind}`,
        body: `\\[\n${bitFields(ieeeResult.bitString, exponentWidth)}\n\\]\n\\[\n\\text{Value} = ${escapeLatex(ieeeResult.value)} \\qquad \\text{${ieeeResult.classification}}\n\\]`
      });
    }

    return latexDocument({ title: 'Floating point toolkit', sections });
  }

  function setMode(next: Mode) {
    mode = next;
    clearResults();
    if (next === 'convert' || next === 'normalise') {
      input = '1011.101';
      from = 2;
    } else if (next === 'custom') {
      input = '6.5';
      customAction = 'encode';
    } else {
      input = '1.5';
      ieeeKind = 'single';
      ieeeAction = 'encode';
    }
  }

  async function run() {
    clearResults();
    try {
      if (mode === 'convert') baseResult = await convertBases(input, from);
      else if (mode === 'normalise') normalResult = await normalise(input, from);
      else if (mode === 'custom') {
        customResult =
          customAction === 'encode'
            ? await encodeCustom(input, exponentBits, mantissaBits)
            : await decodeCustom(input, exponentBits, mantissaBits);
      } else {
        ieeeResult =
          ieeeAction === 'encode'
            ? await encodeIeee(input, ieeeKind)
            : await decodeIeee(input, ieeeKind);
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  preloadFloating();
  void run();
</script>

<form
  class="panel"
  onsubmit={(event: SubmitEvent) => {
    event.preventDefault();
    void run();
  }}
>
  <h2 class="panel-title">Floating point toolkit</h2>

  <Tabs
    ariaLabel="Tool"
    value={mode}
    items={[
      { id: 'convert', label: 'Bases' },
      { id: 'normalise', label: 'Normalise' },
      { id: 'custom', label: 'Binary format' },
      { id: 'ieee', label: 'IEEE 754' }
    ]}
    onChange={(id) => setMode(id as Mode)}
  />

  <div class="fp-controls">
    <div class="field fp-wide">
      <span>{inputLabel}</span>
      <MathField
        bind:value={input}
        format={inputFormat}
        base={inputBase}
        ariaLabel={inputLabel}
        showEnter={false}
        onEnter={() => void run()}
      />
    </div>

    {#if mode === 'convert' || mode === 'normalise'}
      <label class="field">
        <span>Input base</span>
        <select bind:value={from}>
          <option value={2}>Binary (2)</option>
          <option value={8}>Octal (8)</option>
          <option value={10}>Denary (10)</option>
          <option value={16}>Hexadecimal (16)</option>
        </select>
      </label>
    {:else if mode === 'custom'}
      <label class="field">
        <span>Action</span>
        <select bind:value={customAction}>
          <option value="encode">Encode</option>
          <option value="decode">Decode</option>
        </select>
      </label>
      <label class="field">
        <span>Exponent bits</span>
        <input type="number" min="2" max="8" bind:value={exponentBits} />
      </label>
      <label class="field">
        <span>Mantissa bits</span>
        <input type="number" min="2" max="24" bind:value={mantissaBits} />
      </label>
    {:else}
      <label class="field">
        <span>Precision</span>
        <select bind:value={ieeeKind}>
          <option value="single">Single (32-bit)</option>
          <option value="double">Double (64-bit)</option>
        </select>
      </label>
      <label class="field">
        <span>Action</span>
        <select bind:value={ieeeAction}>
          <option value="encode">Encode</option>
          <option value="decode">Decode</option>
        </select>
      </label>
    {/if}
  </div>

  <div class="btn-row">
    <button class="btn" type="submit">{mode === 'convert' ? 'Convert' : mode === 'normalise' ? 'Normalise' : 'Compute'}</button>
  </div>

  <p class="hint">
    {#if mode === 'convert'}
      Enter a value with an optional fractional part, for example <code>1011.101</code> or <code>2D.8</code>. The keypad changes to the selected base.
    {:else if mode === 'normalise'}
      The tool converts to binary and writes the value as a normalised binary mantissa and exponent.
    {:else if mode === 'custom'}
      The custom format uses a sign bit, a biased exponent and an implied leading 1. The reserved top exponent means infinity or NaN.
    {:else}
      Encode a decimal value, or decode exactly 8 hexadecimal digits (single) or 16 (double).
    {/if}
  </p>

  {#if error}<p class="error" role="alert">{error}</p>{/if}
</form>

{#if baseResult}
  <section class="panel" aria-live="polite">
    <StatusLine label="Base conversion" tone="ok" />
    <div class="fp-grid">
      {#each baseResult.outputs as output}
        <div class="fp-card">
          <h3>Base {output.base}</h3>
          <MathBlock latex={`\\texttt{${escapeLatex(output.text)}}`} />
          <span>{output.exact ? 'exact' : 'recurring or truncated'}</span>
        </div>
      {/each}
    </div>
  </section>
{/if}

{#if normalResult}
  <section class="panel" aria-live="polite">
    <StatusLine label="Normalised binary" tone="ok" />
    <div class="fp-form">
      <MathBlock
        latex={`${normalResult.sign === '-' ? '-' : ''}1.${normalResult.fraction || '0'} \\times 2^{${normalResult.exponent}}`}
        display
      />
    </div>
    <dl class="fp-fields">
      <div><dt>Sign</dt><dd>{normalResult.sign === '-' ? 'negative' : 'positive'}</dd></div>
      <div><dt>Exponent</dt><dd>{normalResult.exponent}</dd></div>
      <div><dt>Mantissa</dt><dd><MathText text={`$1.${normalResult.fraction || '0'}$`} /></dd></div>
      <div><dt>Representation</dt><dd>{normalResult.exact ? 'exact' : 'truncated'}</dd></div>
    </dl>
  </section>
{/if}

{#if customResult}
  <section class="panel" aria-live="polite">
    <StatusLine label="Custom binary format" tone="ok" />
    <div class="fp-form">
      <MathBlock latex={bitFields(customResult.bitString, exponentBits)} display />
    </div>
    <dl class="fp-fields">
      <div><dt>Hex</dt><dd><MathBlock latex={`\\texttt{${escapeLatex(customResult.hex)}}`} /></dd></div>
      <div><dt>Sign</dt><dd>{customResult.sign}</dd></div>
      <div><dt>Exponent field</dt><dd>{customResult.exponentField}</dd></div>
      <div><dt>Fraction</dt><dd><MathBlock latex={`\\texttt{${escapeLatex(customResult.fraction)}}`} /></dd></div>
      <div><dt>Value</dt><dd><MathText text={`$${customResult.value}$`} /></dd></div>
    </dl>
    <p class="fp-note">{customResult.note}</p>
  </section>
{/if}

{#if ieeeResult}
  <section class="panel" aria-live="polite">
    <StatusLine
      label={`IEEE 754 ${ieeeResult.kind}`}
      meta={ieeeResult.classification}
      tone="ok"
    />
    <div class="fp-form">
      <MathBlock
        latex={bitFields(ieeeResult.bitString, ieeeResult.kind === 'single' ? 8 : 11)}
        display
      />
    </div>
    <dl class="fp-fields">
      <div><dt>Hex</dt><dd><MathBlock latex={`\\texttt{${escapeLatex(ieeeResult.hex)}}`} /></dd></div>
      <div><dt>Sign</dt><dd>{ieeeResult.sign}</dd></div>
      <div><dt>Exponent field</dt><dd>{ieeeResult.exponentField}</dd></div>
      <div><dt>Fraction</dt><dd><MathBlock latex={`\\texttt{${escapeLatex(ieeeResult.fraction)}}`} /></dd></div>
      <div><dt>Value</dt><dd><MathText text={`$${ieeeResult.value}$`} /></dd></div>
    </dl>
  </section>
{/if}

{#if baseResult || normalResult || customResult || ieeeResult}
  <section class="panel">
    <div class="fp-preview-head">
      <MathLabel latex={'\\LaTeX\\text{ preview}'} fallback="LaTeX preview" />
    </div>
    <div class="latex-preview">
      <LatexPreview latex={exportLatex()} />
    </div>
    <ExportBar latex={exportLatex()} filename="floating-point.tex" />
    <details class="details">
      <summary>LaTeX source</summary>
      <pre class="latex-code">{exportLatex()}</pre>
    </details>
  </section>
{/if}

<style>
  .fp-preview-head {
    margin-bottom: 0.6rem;
    color: var(--fg-dim);
    font-size: 0.9rem;
  }

  .fp-controls {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(9rem, 1fr));
    gap: 0.7rem;
    align-items: end;
    margin-bottom: 0.9rem;
  }

  .fp-wide {
    grid-column: 1 / -1;
  }

  .fp-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr));
    gap: 0.7rem;
    margin-top: 0.8rem;
  }

  .fp-card {
    border: 1px solid var(--line);
    border-radius: var(--radius);
    padding: 0.8rem 0.95rem;
    background: var(--bg-soft);
    display: grid;
    gap: 0.35rem;
  }

  .fp-card h3 {
    margin: 0;
    font-size: 0.72rem;
    font-family: var(--mono);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--fg-faint);
  }

  .fp-card span {
    color: var(--fg-faint);
    font-size: 0.76rem;
  }

  .fp-form {
    margin: 0.8rem 0;
  }

  .fp-fields {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr));
    gap: 0.55rem;
    margin: 0.9rem 0 0;
  }

  .fp-fields > div {
    border: 1px solid var(--line-soft);
    border-radius: 7px;
    padding: 0.5rem 0.65rem;
  }

  .fp-fields dt {
    font-family: var(--mono);
    font-size: 0.68rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--fg-faint);
    margin-bottom: 0.2rem;
  }

  .fp-fields dd {
    margin: 0;
    font-family: var(--mono);
    font-size: 0.82rem;
    color: var(--fg);
    overflow-wrap: anywhere;
  }

  .fp-note {
    margin: 0.8rem 0 0;
    color: var(--fg-dim);
    font-size: 0.86rem;
  }
</style>
