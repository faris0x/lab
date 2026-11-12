<script lang="ts">
  let {
    step,
    last,
    label,
    previous = '← Previous',
    next = 'Next →',
    onStep
  }: {
    step: number;
    last: number;
    label: string;
    previous?: string;
    next?: string;
    onStep: (step: number) => void;
  } = $props();
</script>

<div class="step-controls">
  <button
    class="btn btn-ghost btn-small"
    type="button"
    disabled={step === 0}
    onclick={() => onStep(Math.max(0, step - 1))}>{previous}</button
  >
  <input
    type="range"
    min="0"
    max={last}
    value={step}
    aria-label={label}
    oninput={(event) => onStep(Number((event.currentTarget as HTMLInputElement).value))}
  />
  <button
    class="btn btn-ghost btn-small"
    type="button"
    disabled={step === last}
    onclick={() => onStep(Math.min(last, step + 1))}>{next}</button
  >
  <span class="step-count">{label}</span>
</div>
