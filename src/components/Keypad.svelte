<script lang="ts">
  import MathLabel from './MathLabel.svelte';
  import type { Base, MathFormat } from '../lib/math-input';

  type Action = 'insert' | 'backspace' | 'clear' | 'enter';

  interface Key {
    tex?: string;
    insert?: string;
    action?: Action;
    wide?: boolean;
    tone?: 'accent' | 'ghost';
    aria?: string;
    erase?: boolean;
  }

  let {
    format = 'number',
    base = 10,
    variables = ['x', 'y', 'z'],
    showEnter = true,
    onInsert,
    onBackspace,
    onClear,
    onEnter
  }: {
    format?: MathFormat;
    base?: Base;
    variables?: string[];
    showEnter?: boolean;
    onInsert: (latex: string) => void;
    onBackspace: () => void;
    onClear: () => void;
    onEnter?: () => void;
  } = $props();

  const digitsForBase = (value: Base): string[] =>
    '0123456789ABCDEF'.slice(0, value).split('');

  function numericKeys(): Key[] {
    if (format === 'base') {
      const keys: Key[] = digitsForBase(base).map((digit) => ({
        tex: base === 16 ? `\\mathrm{${digit}}` : digit,
        insert: base === 16 ? `\\mathrm{${digit}}` : digit
      }));
      keys.push({ tex: '.', insert: '.', aria: 'point' });
      keys.push({ tex: '\\pm', insert: '-', aria: 'negate' });
      return keys;
    }
    return [
      { tex: '7', insert: '7' },
      { tex: '8', insert: '8' },
      { tex: '9', insert: '9' },
      { tex: '\\div', insert: '\\div ', aria: 'divide' },
      { tex: '4', insert: '4' },
      { tex: '5', insert: '5' },
      { tex: '6', insert: '6' },
      { tex: '\\times', insert: '\\times ', aria: 'multiply' },
      { tex: '1', insert: '1' },
      { tex: '2', insert: '2' },
      { tex: '3', insert: '3' },
      { tex: '-', insert: '-', aria: 'minus' },
      { tex: '0', insert: '0' },
      { tex: '.', insert: '.', aria: 'point' },
      { tex: '\\pm', insert: '-', aria: 'negate' },
      { tex: '+', insert: '+', aria: 'plus' }
    ];
  }

  function logicKeys(): Key[] {
    return [
      { tex: '\\neg', insert: '\\neg ', aria: 'not' },
      { tex: '\\land', insert: '\\land ', aria: 'and' },
      { tex: '\\lor', insert: '\\lor ', aria: 'or' },
      { tex: '\\oplus', insert: '\\oplus ', aria: 'xor' },
      { tex: '\\to', insert: '\\to ', aria: 'implies' },
      { tex: '\\leftrightarrow', insert: '\\leftrightarrow ', aria: 'equivalent' },
      { tex: '(', insert: '(' },
      { tex: ')', insert: ')' },
      { tex: '\\top', insert: '\\top', aria: 'true' },
      { tex: '\\bot', insert: '\\bot', aria: 'false' },
      { tex: 'p', insert: 'p' },
      { tex: 'q', insert: 'q' },
      { tex: 'r', insert: 'r' }
    ];
  }

  function expressionKeys(): Key[] {
    return [
      ...variables.map((variable) => ({ tex: variable, insert: variable })),
      { tex: 'x^{2}', insert: '^{2}', aria: 'squared' },
      { tex: 'x^{y}', insert: '^{#?}', aria: 'power' },
      { tex: '\\sqrt{x}', insert: '\\sqrt{#0}', aria: 'square root' },
      { tex: '(', insert: '(' },
      { tex: ')', insert: ')' },
      { tex: '\\div', insert: '\\div ', aria: 'divide' },
      { tex: '\\times', insert: '\\times ', aria: 'multiply' },
      { tex: '-', insert: '-', aria: 'minus' },
      { tex: '+', insert: '+', aria: 'plus' },
      { tex: '7', insert: '7' },
      { tex: '8', insert: '8' },
      { tex: '9', insert: '9' },
      { tex: '4', insert: '4' },
      { tex: '5', insert: '5' },
      { tex: '6', insert: '6' },
      { tex: '1', insert: '1' },
      { tex: '2', insert: '2' },
      { tex: '3', insert: '3' },
      { tex: '0', insert: '0' },
      { tex: '.', insert: '.', aria: 'point' },
      { tex: '\\pm', insert: '-', aria: 'negate' }
    ];
  }

  const keys = $derived(
    format === 'logic'
      ? logicKeys()
      : format === 'expression'
        ? expressionKeys()
        : numericKeys()
  );

  function press(key: Key) {
    const action = key.action ?? 'insert';
    if (action === 'backspace') return onBackspace();
    if (action === 'clear') return onClear();
    if (action === 'enter') return onEnter?.();
    onInsert(key.insert ?? '');
  }
</script>

<div class="keypad" role="group" aria-label="Maths keypad" onpointerdown={(event) => event.preventDefault()}>
  {#if format === 'number' || format === 'expression'}
    <button class="key key-wide" type="button" onclick={() => onInsert('\\frac{#0}{#?}')} aria-label="fraction">
      <MathLabel latex={'\\frac{a}{b}'} />
    </button>
    <button class="key" type="button" onclick={() => onInsert('^{#?}')} aria-label="power">
      <MathLabel latex={'x^{y}'} />
    </button>
    <button class="key" type="button" onclick={() => onInsert('\\sqrt{#0}')} aria-label="square root">
      <MathLabel latex={'\\sqrt{x}'} />
    </button>
    <button class="key" type="button" onclick={() => onInsert('\\left(#0\\right)')} aria-label="parentheses">
      <MathLabel latex={'(\\;)'} />
    </button>
  {/if}

  {#each keys as key}
    <button
      class="key"
      class:key-accent={key.tone === 'accent'}
      type="button"
      onclick={() => press(key)}
      aria-label={key.aria ?? key.tex ?? ''}
    >
      {#if key.tex}<MathLabel latex={key.tex} />{/if}
    </button>
  {/each}

  <button class="key key-ghost" type="button" onclick={onBackspace} aria-label="backspace">
    <span class="erase" aria-hidden="true">⌫</span>
  </button>
  <button class="key key-ghost" type="button" onclick={onClear} aria-label="clear">
    <MathLabel latex={'\\mathrm{C}'} />
  </button>
  {#if showEnter}
    <button class="key key-accent key-wide" type="button" onclick={() => onEnter?.()} aria-label="run">
      <MathLabel latex={'\\text{Run}'} />
    </button>
  {/if}
</div>

<style>
  .keypad {
    display: grid;
    grid-template-columns: repeat(4, minmax(2.5rem, 1fr));
    gap: 0.3rem;
    margin-top: 0.55rem;
    max-width: 24rem;
    touch-action: manipulation;
  }

  .key {
    font: inherit;
    font-size: 0.95rem;
    font-weight: 500;
    min-height: 2.5rem;
    padding: 0.3rem 0.2rem;
    border: 1px solid var(--line);
    border-radius: 7px;
    background: var(--bg-soft);
    color: var(--fg);
    cursor: pointer;
    user-select: none;
  }

  .key:hover {
    border-color: var(--accent);
  }

  .key:active {
    background: var(--accent-soft);
  }

  .key-wide {
    grid-column: span 2;
  }

  .key-ghost {
    color: var(--fg-dim);
  }

  .key-accent {
    background: var(--accent);
    border-color: transparent;
    color: var(--bg);
    font-weight: 700;
  }

  .erase {
    font-family: 'Segoe UI Symbol', 'Noto Sans Symbols 2', 'Noto Sans Symbols',
      'DejaVu Sans', sans-serif;
    font-size: 1.15rem;
    line-height: 1;
  }

  @media (max-width: 640px) {
    .keypad {
      max-width: none;
      grid-template-columns: repeat(4, minmax(2.75rem, 1fr));
    }

    .key {
      min-height: 2.85rem;
    }
  }
</style>
