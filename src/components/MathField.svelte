<script lang="ts">
  import { onMount } from 'svelte';
  import 'mathlive/fonts.css';
  import Keypad from './Keypad.svelte';
  import {
    canonicalToLatex,
    latexToCanonical,
    type Base,
    type MathFormat
  } from '../lib/math-input';
  import {
    activateField,
    deactivateField,
    type MathFieldHandle
  } from '../lib/active-field';

  let {
    value = $bindable(''),
    format = 'number',
    base = 10,
    ariaLabel = 'Mathematical input',
    placeholder = '',
    keypad = true,
    showEnter = true,
    compact = false,
    autofocus = false,
    onValueChange,
    onEnter
  }: {
    value?: string;
    format?: MathFormat;
    base?: Base;
    ariaLabel?: string;
    placeholder?: string;
    keypad?: boolean;
    showEnter?: boolean;
    compact?: boolean;
    autofocus?: boolean;
    onValueChange?: (value: string) => void;
    onEnter?: () => void;
  } = $props();

  let host: HTMLDivElement;
  let field: any = null;
  let ready = $state(false);
  let lastCanonical = value;

  const api: MathFieldHandle = {
    insert(latex: string) {
      if (!field) return;
      field.focus();
      field.insert(latex);
    },
    backspace() {
      if (!field) return;
      field.focus();
      field.executeCommand('deleteBackward');
    },
    clear() {
      if (!field) return;
      field.focus();
      field.value = '';
      lastCanonical = '';
      value = '';
      onValueChange?.('');
    },
    focus() {
      field?.focus();
    },
    getValue() {
      return lastCanonical;
    }
  };

  function readField() {
    if (!field) return;
    const canonical = latexToCanonical(field.getValue('latex'), format, base);
    if (canonical === lastCanonical) return;
    lastCanonical = canonical;
    value = canonical;
    onValueChange?.(canonical);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      onEnter?.();
    }
  }

  onMount(() => {
    let alive = true;

    (async () => {
      const mathlive = await import('mathlive');
      const MathfieldElement = (mathlive as any).MathfieldElement ?? (mathlive as any).default?.MathfieldElement;
      if (MathfieldElement) MathfieldElement.locale = 'en';
      if (!alive || !host) return;

      const element = document.createElement('math-field');
      element.setAttribute('aria-label', ariaLabel);
      element.setAttribute('math-virtual-keyboard-policy', 'manual');
      element.mathVirtualKeyboardPolicy = 'manual';
      element.smartFence = true;
      element.smartMode = true;
      element.placeholder = placeholder;
      element.value = canonicalToLatex(value, format, base);
      element.addEventListener('input', readField);
      element.addEventListener('focus', () => activateField(api));
      element.addEventListener('blur', () => deactivateField(api));
      element.addEventListener('keydown', handleKeydown as EventListener);
      host.appendChild(element);
      field = element;
      ready = true;
      if (autofocus) element.focus();
    })();

    return () => {
      alive = false;
      if (field) {
        field.removeEventListener('input', readField);
        field.removeEventListener('keydown', handleKeydown as EventListener);
        deactivateField(api);
        field.remove();
        field = null;
      }
    };
  });

  $effect(() => {
    const next = value;
    if (!field || next === lastCanonical) return;
    lastCanonical = next;
    field.value = canonicalToLatex(next, format, base);
  });
</script>

<div class="mathfield" class:mathfield-ready={ready} class:mathfield-compact={compact}>
  <div class="mathfield-host" bind:this={host}></div>
</div>

{#if keypad}
  <Keypad
    {format}
    {base}
    onInsert={(latex) => api.insert(latex)}
    onBackspace={() => api.backspace()}
    onClear={() => api.clear()}
    onEnter={() => onEnter?.()}
    {showEnter}
  />
{/if}

<style>
  .mathfield {
    display: flex;
    align-items: center;
    min-height: 2.65rem;
    padding: 0.2rem 0.55rem;
    border: 1px solid var(--line);
    border-radius: 7px;
    background: var(--bg-soft);
    transition: border-color 0.15s var(--ease);
  }

  .mathfield:hover {
    border-color: var(--fg-faint);
  }

  .mathfield:focus-within {
    border-color: var(--accent);
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .mathfield-host {
    flex: 1;
    min-width: 0;
  }

  .mathfield-compact {
    min-height: 2.2rem;
    padding: 0.05rem 0.3rem;
    border-radius: 5px;
  }

  .mathfield-compact :global(math-field) {
    font-size: 0.95rem;
  }

  .mathfield :global(math-field) {
    display: block;
    width: 100%;
    min-width: 0;
    border: 0;
    outline: 0;
    background: transparent;
    color: var(--fg);
    font-size: 1.08rem;
    --caret-color: var(--accent);
    --selection-background-color: var(--accent-soft);
    --selection-color: var(--fg);
    --contains-highlight-background-color: transparent;
    --smart-fence-color: var(--accent);
    --smart-fence-opacity: 0.55;
  }

  .mathfield :global(math-field::part(menu-toggle)),
  .mathfield :global(math-field::part(virtual-keyboard-toggle)) {
    display: none;
  }
</style>
