<script lang="ts">
  let {
    label,
    meta = '',
    tone = 'neutral'
  }: {
    label: string;
    meta?: string;
    tone?: 'ok' | 'warn' | 'neutral';
  } = $props();
</script>

<div class="status-line">
  <span
    class="status"
    class:status-ok={tone === 'ok'}
    class:status-warn={tone === 'warn'}>{label}</span
  >
  {#if meta}<span class="step-count">{meta}</span>{/if}
</div>
