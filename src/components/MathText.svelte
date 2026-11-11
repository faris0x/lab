<script lang="ts">
  import { renderMathText } from '../lib/math';
  import { localizeProse } from '../lib/localize';
  import { lang } from '../lib/lang';

  let { text }: { text: string } = $props();
  let html = $state('');

  const shown = $derived($lang === 'ar' ? localizeProse(text) : text);

  $effect(() => {
    let active = true;
    const source = shown;
    html = '';
    renderMathText(source)
      .then((output) => {
        if (active) html = output;
      })
      .catch(() => {});
    return () => {
      active = false;
    };
  });
</script>

{#if html}
  {@html html}
{:else}
  {shown.replace(/\$/g, '')}
{/if}
