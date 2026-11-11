<script lang="ts">
  import { scaleLinear } from 'd3-scale';
  import MathBlock from './Math.svelte';
  import type { ArgandCircle, ArgandPoint } from '../lib/complex';
  import { arNumeric } from '../lib/localize';
  import { lang } from '../lib/lang';

  let {
    points,
    circles
  }: { points: ArgandPoint[]; circles: ArgandCircle[] } = $props();

  let ar = $state(false);
  $effect(() => {
    ar = $lang === 'ar';
  });
  const size = 460;
  const padding = 46;

  const maxAbs = $derived.by(() => {
    let max = 1;
    for (const point of points) {
      max = Math.max(max, Math.abs(point.x), Math.abs(point.y));
    }
    for (const circle of circles) {
      max = Math.max(max, circle.radius);
    }
    return Math.max(1, max * 1.08);
  });

  const xScale = $derived(
    scaleLinear()
      .domain([-maxAbs, maxAbs])
      .nice(5)
      .range([padding, size - padding])
  );
  const yScale = $derived(
    scaleLinear()
      .domain(xScale.domain())
      .range([size - padding, padding])
  );
  const ticks = $derived(xScale.ticks(5));
  const zeroX = $derived(xScale(0));
  const zeroY = $derived(yScale(0));

  function radiusPx(radius: number): number {
    return Math.abs(xScale(radius) - xScale(0));
  }

  function tickLabel(value: number): string {
    const text = xScale.tickFormat(5)(value);
    return ar ? arNumeric(text) : text;
  }

  function fmt4(value: number): string {
    if (!Number.isFinite(value)) return '0';
    let text = value.toFixed(4);
    if (text.includes('.')) {
      text = text.replace(/0+$/, '').replace(/\.$/, '');
    }
    return text === '-0' ? '0' : text;
  }
</script>

<div class="argand">
  <svg
    viewBox={`0 0 ${size} ${size}`}
    role="img"
    aria-label={ar ? 'مخطط أرغاند بمحورين حقيقي وتخيلي' : 'Argand diagram with real and imaginary axes'}
  >
    <defs>
      <marker
        id="argand-arrow"
        viewBox="0 0 8 8"
        refX="7"
        refY="4"
        markerWidth="7"
        markerHeight="7"
        orient="auto-start-reverse"
      >
        <path d="M 0 0 L 8 4 L 0 8 z" class="argand-arrow-head" />
      </marker>
    </defs>

    <rect class="argand-bg" x="0" y="0" width={size} height={size} />

    {#each ticks as tick}
      {#if Math.abs(tick) > 1e-9}
        <line
          class="argand-grid"
          x1={xScale(tick)}
          y1={padding}
          x2={xScale(tick)}
          y2={size - padding}
        />
        <line
          class="argand-grid"
          x1={padding}
          y1={yScale(tick)}
          x2={size - padding}
          y2={yScale(tick)}
        />
      {/if}
    {/each}

    <line
      class="argand-axis"
      x1={padding}
      y1={zeroY}
      x2={size - 8}
      y2={zeroY}
      marker-end="url(#argand-arrow)"
    />
    <line
      class="argand-axis"
      x1={zeroX}
      y1={size - padding}
      x2={zeroX}
      y2="8"
      marker-end="url(#argand-arrow)"
    />

    <text class="argand-axis-label" x={size - 8} y={zeroY - 10} text-anchor="end"
      >Re(z)</text
    >
    <text class="argand-axis-label" x={zeroX + 10} y="18">Im(z)</text>

    {#each ticks as tick}
      {#if Math.abs(tick) > 1e-9}
        <text
          class="argand-tick"
          x={xScale(tick)}
          y={size - padding + 18}
          text-anchor="middle">{tickLabel(tick)}</text
        >
        <text
          class="argand-tick"
          x={padding - 8}
          y={yScale(tick) + 4}
          text-anchor="end">{tickLabel(tick)}</text
        >
      {/if}
    {/each}

    <text class="argand-tick" x={zeroX - 6} y={zeroY + 16} text-anchor="end">{ar ? '٠' : '0'}</text>

    {#each circles as circle}
      <circle
        class="argand-circle"
        cx={zeroX}
        cy={zeroY}
        r={radiusPx(circle.radius)}
      />
    {/each}

    {#each points as point}
      {#if point.kind !== 'root'}
        <line
          class="argand-spoke"
          class:argand-spoke-result={point.kind === 'result'}
          x1={zeroX}
          y1={zeroY}
          x2={xScale(point.x)}
          y2={yScale(point.y)}
        />
      {/if}
      <circle
        class:argand-point-input={point.kind === 'input'}
        class:argand-point-result={point.kind === 'result'}
        class:argand-point-root={point.kind === 'root'}
        cx={xScale(point.x)}
        cy={yScale(point.y)}
        r="5"
      >
        <title>{point.label} ({ar ? arNumeric(fmt4(point.x)) : fmt4(point.x)}, {ar ? arNumeric(fmt4(point.y)) : fmt4(point.y)})</title>
      </circle>
    {/each}
  </svg>

  {#each points as point}
    <span
      class="argand-tag"
      class:argand-tag-input={point.kind === 'input'}
      class:argand-tag-result={point.kind === 'result'}
      class:argand-tag-root={point.kind === 'root'}
      style={`left: ${(xScale(point.x) / size) * 100}%; top: ${(yScale(point.y) / size) * 100}%; transform: ${point.kind === 'input' || point.x < 0 ? 'translate(-10px, -50%) translateX(-100%)' : 'translate(8px, -50%)'}`}
    >
      <MathBlock latex={point.labelLatex} fallback={point.label} />
    </span>
  {/each}
</div>
