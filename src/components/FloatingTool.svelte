<script lang="ts">
  import {
    convertBases,
    decodeCustom,
    decodeIeee,
    encodeCustom,
    encodeIeee,
    normalise,
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
  let copyState = $state('');

  function clearResults() {
    copyState = '';
    baseResult = null;
    normalResult = null;
    customResult = null;
    ieeeResult = null;
    error = '';
  }

  function exportLatex(): string {
    if (!baseResult && !normalResult && !customResult && !ieeeResult) return '';
    const sections: string[] = [];
    if (baseResult) {
      const rows = baseResult.outputs.map(
        (output) =>
          `Base ${output.base}: \\texttt{${output.text.replace(/[_^]/g, (character) => `\\${character}`)}}`
      );
      sections.push(`\\section*{Base conversion}\n${rows.join('\n')}`);
    }
    if (normalResult) {
      const binary = normalResult.binary.replace(/×/g, '\\times ');
      sections.push(`\\section*{Normalised binary}\n\\[${binary}\\]`);
    }
    if (customResult) {
      sections.push(
        `\\section*{Custom binary format}\nBinary: \\texttt{${customResult.bitString}}\\\\\nHex: \\texttt{${customResult.hex}}\\\\\nValue: ${customResult.value}`
      );
    }
    if (ieeeResult) {
      sections.push(
        `\\section*{IEEE 754}\nHex: \\texttt{${ieeeResult.hex}}\\\\\nBinary: \\texttt{${ieeeResult.bitString}}\\\\\nValue: ${ieeeResult.value}`
      );
    }
    return `\\documentclass[11pt]{article}
\\usepackage[a4paper,margin=2.2cm]{geometry}
\\usepackage{amsmath,amssymb}
\\setlength{\\parindent}{0pt}
\\begin{document}
\\section*{Floating point toolkit}
${sections.join('\n\n')}
\\end{document}
`;
  }

  async function copyLatex() {
    const text = exportLatex();
    if (!text) return;
    try {
      await navigator.clipboard.writeText(text);
      copyState = 'Copied';
    } catch {
      copyState = 'Copy failed';
    }
    window.setTimeout(() => {
      copyState = '';
    }, 1600);
  }

  function downloadLatex() {
    const text = exportLatex();
    if (!text) return;
    const blob = new Blob([text], { type: 'application/x-tex' });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = 'floating-point.tex';
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    window.setTimeout(() => URL.revokeObjectURL(url), 1000);
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

  function run() {
    clearResults();
    try {
      if (mode === 'convert') baseResult = convertBases(input, from);
      else if (mode === 'normalise') normalResult = normalise(input, from);
      else if (mode === 'custom') {
        customResult =
          customAction === 'encode'
            ? encodeCustom(input, exponentBits, mantissaBits)
            : decodeCustom(input, exponentBits, mantissaBits);
      } else {
        ieeeResult =
          ieeeAction === 'encode'
            ? encodeIeee(input, ieeeKind)
            : decodeIeee(input, ieeeKind);
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
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
  <h2 class="panel-title">Floating point toolkit</h2>

  <div class="tabs fp-modes" role="tablist" aria-label="Tool">
    <button class="tab" type="button" role="tab" aria-selected={mode === 'convert'} onclick={() => setMode('convert')}>Bases</button>
    <button class="tab" type="button" role="tab" aria-selected={mode === 'normalise'} onclick={() => setMode('normalise')}>Normalise</button>
    <button class="tab" type="button" role="tab" aria-selected={mode === 'custom'} onclick={() => setMode('custom')}>Binary format</button>
    <button class="tab" type="button" role="tab" aria-selected={mode === 'ieee'} onclick={() => setMode('ieee')}>IEEE 754</button>
  </div>

  <div class="fp-controls">
    <label class="field fp-wide">
      <span>{mode === 'custom' && customAction === 'decode' ? 'Bit pattern' : mode === 'ieee' && ieeeAction === 'decode' ? 'Hexadecimal bits' : 'Value'}</span>
      <input bind:value={input} spellcheck="false" autocomplete="off" />
    </label>

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
      Enter a value with an optional fractional part, for example <code>1011.101</code> or <code>2D.8</code>.
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
  <section class="panel">
    <div class="status-line"><span class="status status-ok">Base conversion</span></div>
    <div class="fp-grid">
      {#each baseResult.outputs as output}
        <div class="fp-card">
          <h3>Base {output.base}</h3>
          <code>{output.text}</code>
          <span>{output.exact ? 'exact' : 'recurring or truncated'}</span>
        </div>
      {/each}
    </div>
  </section>
{/if}

{#if normalResult}
  <section class="panel">
    <div class="status-line"><span class="status status-ok">Normalised binary</span></div>
    <p class="fp-form">{normalResult.binary}</p>
    <dl class="fp-fields">
      <div><dt>Sign</dt><dd>{normalResult.sign || 'positive'}</dd></div>
      <div><dt>Exponent</dt><dd>{normalResult.exponent}</dd></div>
      <div><dt>Mantissa</dt><dd>1.{normalResult.fraction || '0'}</dd></div>
      <div><dt>Representation</dt><dd>{normalResult.exact ? 'exact' : 'truncated'}</dd></div>
    </dl>
  </section>
{/if}

{#if customResult}
  <section class="panel">
    <div class="status-line"><span class="status status-ok">Custom binary format</span></div>
    <dl class="fp-fields">
      <div><dt>Binary</dt><dd><code>{customResult.bitString}</code></dd></div>
      <div><dt>Hex</dt><dd><code>{customResult.hex}</code></dd></div>
      <div><dt>Sign</dt><dd>{customResult.sign}</dd></div>
      <div><dt>Exponent field</dt><dd>{customResult.exponentField}</dd></div>
      <div><dt>Fraction</dt><dd><code>{customResult.fraction}</code></dd></div>
      <div><dt>Value</dt><dd>{customResult.value}</dd></div>
    </dl>
    <p class="fp-note">{customResult.note}</p>
  </section>
{/if}

{#if ieeeResult}
  <section class="panel">
    <div class="status-line">
      <span class="status status-ok">IEEE 754 {ieeeResult.kind}</span>
      <span class="step-count">{ieeeResult.classification}</span>
    </div>
    <dl class="fp-fields">
      <div><dt>Hex</dt><dd><code>{ieeeResult.hex}</code></dd></div>
      <div><dt>Binary</dt><dd><code>{ieeeResult.bitString}</code></dd></div>
      <div><dt>Sign</dt><dd>{ieeeResult.sign}</dd></div>
      <div><dt>Exponent field</dt><dd>{ieeeResult.exponentField}</dd></div>
      <div><dt>Fraction</dt><dd><code>{ieeeResult.fraction}</code></dd></div>
      <div><dt>Value</dt><dd>{ieeeResult.value}</dd></div>
    </dl>
  </section>
{/if}

{#if baseResult || normalResult || customResult || ieeeResult}
  <section class="panel">
    <div class="btn-row" style="margin-top:0">
      <button class="btn btn-ghost btn-small" type="button" onclick={copyLatex}
        >{copyState || 'Copy full'} LaTeX</button
      >
      <button class="btn btn-ghost btn-small" type="button" onclick={downloadLatex}
        >Download .tex</button
      >
    </div>
    <details class="details">
      <summary>LaTeX source</summary>
      <pre class="latex-code">{exportLatex()}</pre>
    </details>
  </section>
{/if}

<style>
  .fp-modes {
    margin-bottom: 0.9rem;
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

  .fp-card code,
  .fp-form {
    color: var(--accent);
    font-family: var(--mono);
    font-size: 1rem;
    overflow-wrap: anywhere;
  }

  .fp-card span {
    color: var(--fg-faint);
    font-size: 0.76rem;
  }

  .fp-form {
    font-size: 1.15rem;
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
