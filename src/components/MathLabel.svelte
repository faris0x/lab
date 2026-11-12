<script lang="ts">
  import { renderMath } from '../lib/math';

  let { latex, fallback = '' }: { latex: string; fallback?: string } = $props();
  let html = $state('');

  $effect(() => {
    let active = true;
    renderMath(latex, false)
      .then((output) => {
        if (active) html = output;
      })
      .catch(() => {});
    return () => {
      active = false;
    };
  });
</script>

<span class="key-math"
  >{#if html}{@html html}{:else}{fallback || latex}{/if}</span
>

<style>
  .key-math {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
  }

  .key-math :global(.katex) {
    font-size: 0.98em;
  }
</style>
