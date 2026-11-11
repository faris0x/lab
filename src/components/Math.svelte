<script lang="ts">
  import { renderMath } from '../lib/math';
  import { lang } from '../lib/lang';

  let {
    latex,
    display = false,
    fallback = ''
  }: { latex: string; display?: boolean; fallback?: string } = $props();

  let html = $state('');

  $effect(() => {
    let active = true;
    const activeLang = $lang;
    const source = activeLang === 'ar' ? latex : latex;
    html = '';
    renderMath(source, display)
      .then((output) => {
        if (active) html = output;
      })
      .catch(() => {});
    return () => {
      active = false;
    };
  });
</script>

{#if display}
  <div class="math-block">
    {#if html}
      {@html html}
    {:else}
      <span class="math-pending">{fallback || latex}</span>
    {/if}
  </div>
{:else}
  <span class="math-inline">
    {#if html}
      {@html html}
    {:else}
      <span class="math-pending">{fallback || latex}</span>
    {/if}
  </span>
{/if}
