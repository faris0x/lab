<script lang="ts">
  import MathBlock from './Math.svelte';
  import LatexText from './LatexText.svelte';
  import { parseLatexDocument } from '../lib/latex-preview';

  let { latex }: { latex: string } = $props();
  const blocks = $derived(parseLatexDocument(latex));
</script>

<div class="latex-document">
  {#each blocks as block}
    {#if block.kind === 'heading'}
      <svelte:element this={`h${block.level + 2}`} class="latex-heading">
        {block.text}
      </svelte:element>
    {:else if block.kind === 'math'}
      <MathBlock latex={block.latex} display />
    {:else if block.kind === 'list'}
      <svelte:element this={block.ordered ? 'ol' : 'ul'} class="latex-list">
        {#each block.items as parts}
          <li>
            {#each parts as part}
              {#if part.kind === 'math'}
                <MathBlock latex={part.latex} display />
              {:else if part.kind === 'text'}
                <LatexText text={part.text} />
              {/if}
            {/each}
          </li>
        {/each}
      </svelte:element>
    {:else}
      <p><LatexText text={block.text} /></p>
    {/if}
  {/each}
</div>

<style>
  .latex-document {
    display: grid;
    gap: 0.65rem;
    color: var(--fg-dim);
    font-family: 'KaTeX_Main', 'Latin Modern Roman', 'Times New Roman', serif;
    font-size: 1.02rem;
    line-height: 1.55;
  }

  .latex-document :global(.math-block) {
    margin: 0.2rem 0;
  }

  .latex-document :global(> p) {
    margin: 0;
  }

  .latex-heading {
    margin: 0.5rem 0 0.1rem;
    color: var(--fg);
    font-family: 'KaTeX_Main', 'Latin Modern Roman', 'Times New Roman', serif;
    font-size: 1.05rem;
    letter-spacing: 0;
  }

  .latex-list {
    margin: 0;
    padding-left: 1.2rem;
  }

  .latex-list :global(li) {
    margin-bottom: 0.35rem;
  }

  .latex-list :global(li)::marker {
    color: var(--accent);
  }
</style>
