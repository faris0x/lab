<script lang="ts">
  import MathLabel from './MathLabel.svelte';

  export interface TabItem {
    id: string;
    label: string;
    latex?: string;
  }

  let {
    items,
    value,
    onChange,
    ariaLabel = 'View'
  }: {
    items: TabItem[];
    value: string;
    onChange: (id: string) => void;
    ariaLabel?: string;
  } = $props();

  function move(event: KeyboardEvent, index: number) {
    if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
    event.preventDefault();
    let next = index;
    if (event.key === 'ArrowLeft') next = (index - 1 + items.length) % items.length;
    else if (event.key === 'ArrowRight') next = (index + 1) % items.length;
    else if (event.key === 'Home') next = 0;
    else next = items.length - 1;
    onChange(items[next].id);
    const tabs = (event.currentTarget as HTMLElement).parentElement?.querySelectorAll<HTMLButtonElement>('[role="tab"]');
    tabs?.[next]?.focus();
  }
</script>

<div class="tabs" role="tablist" aria-label={ariaLabel}>
  {#each items as item, index}
    <button
      class="tab"
      type="button"
      role="tab"
      id={`tab-${item.id}`}
      aria-selected={value === item.id}
      tabindex={value === item.id ? 0 : -1}
      onclick={() => onChange(item.id)}
      onkeydown={(event) => move(event, index)}
      aria-label={item.latex ? `LaTeX ${item.label}` : undefined}
    >
      {#if item.latex}
        <MathLabel latex={item.latex} fallback="LaTeX" />
        <span>{item.label}</span>
      {:else}
        {item.label}
      {/if}
    </button>
  {/each}
</div>
