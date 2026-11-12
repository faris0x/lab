<script lang="ts">
  import LatexLogo from './LatexLogo.svelte';

  let {
    latex,
    filename = 'solution.tex',
    copyLabel = 'Copy full',
    downloadLabel = 'Download .tex'
  }: {
    latex: string;
    filename?: string;
    copyLabel?: string;
    downloadLabel?: string;
  } = $props();

  let copyState = $state('');

  async function copy() {
    if (!latex) return;
    try {
      await navigator.clipboard.writeText(latex);
      copyState = 'Copied';
    } catch {
      copyState = 'Copy failed';
    }
    window.setTimeout(() => {
      copyState = '';
    }, 1600);
  }

  function download() {
    if (!latex) return;
    const blob = new Blob([latex], { type: 'application/x-tex' });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = filename;
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    window.setTimeout(() => URL.revokeObjectURL(url), 1000);
  }
</script>

<div class="btn-row export-bar" style="margin-top:0">
  <button class="btn btn-ghost btn-small" type="button" onclick={copy}>
    {copyState || copyLabel} <LatexLogo />
  </button>
  <button class="btn btn-ghost btn-small" type="button" onclick={download}>
    {downloadLabel}
  </button>
</div>
