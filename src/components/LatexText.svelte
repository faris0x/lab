<script lang="ts">
  import { renderMath } from '../lib/math';

  let { text }: { text: string } = $props();
  let html = $state('');

  function escapeHtml(value: string): string {
    return value.replace(/[&<>"']/g, (character) => {
      switch (character) {
        case '&':
          return '&amp;';
        case '<':
          return '&lt;';
        case '>':
          return '&gt;';
        case '"':
          return '&quot;';
        default:
          return '&#39;';
      }
    });
  }

  function readGroup(source: string, start: number): { body: string; end: number } | null {
    if (source[start] !== '{') return null;
    let depth = 0;
    for (let index = start; index < source.length; index += 1) {
      const character = source[index];
      if (character === '\\') {
        index += 1;
        continue;
      }
      if (character === '{') depth += 1;
      else if (character === '}') {
        depth -= 1;
        if (depth === 0) return { body: source.slice(start + 1, index), end: index + 1 };
      }
    }
    return null;
  }

  const symbols = new Set([
    '\\to', '\\rightarrow', '\\leftarrow', '\\leftrightarrow',
    '\\pm', '\\mp', '\\times', '\\div', '\\cdot', '\\infty',
    '\\le', '\\ge', '\\ne', '\\approx', '\\equiv',
    '\\pi', '\\alpha', '\\beta', '\\gamma', '\\theta', '\\lambda',
    '\\Sigma', '\\Pi', '\\sum', '\\prod', '\\in', '\\notin',
    '\\subset', '\\subseteq', '\\cup', '\\cap', '\\emptyset',
    '\\ldots', '\\dots', '\\cdots', '\\quad', '\\qquad'
  ]);

  const wrappers: { command: string; open: string; close: string }[] = [
    { command: '\\textbf', open: '<strong>', close: '</strong>' },
    { command: '\\textit', open: '<em>', close: '</em>' },
    { command: '\\emph', open: '<em>', close: '</em>' },
    { command: '\\texttt', open: '<code>', close: '</code>' },
    { command: '\\underline', open: '<u>', close: '</u>' },
    { command: '\\text', open: '', close: '' },
    { command: '\\mathrm', open: '', close: '' },
    { command: '\\textrm', open: '', close: '' }
  ];

  async function build(source: string): Promise<string> {
    let output = '';
    let index = 0;

    while (index < source.length) {
      const rest = source.slice(index);

      if (rest.startsWith('\\LaTeX')) {
        output += 'LaTeX';
        index += '\\LaTeX'.length;
        continue;
      }

      const wrapper = wrappers.find((entry) => rest.startsWith(`${entry.command}{`));
      if (wrapper) {
        const start = index + wrapper.command.length;
        const group = readGroup(source, start);
        if (group) {
          output += `${wrapper.open}${await build(group.body)}${wrapper.close}`;
          index = group.end;
          continue;
        }
      }

      if (rest.startsWith('$')) {
        const end = source.indexOf('$', index + 1);
        if (end !== -1) {
          const body = source.slice(index + 1, end);
          try {
            output += await renderMath(body, false);
          } catch {
            output += escapeHtml(`$${body}$`);
          }
          index = end + 1;
          continue;
        }
      }

      if (rest.startsWith('\\\\')) {
        output += '<br />';
        index += 2;
        continue;
      }

      if (rest.startsWith('\\begin{') || rest.startsWith('\\end{')) {
        const brace = rest.indexOf('{');
        const group = readGroup(rest, brace);
        index += group ? group.end : rest.indexOf('}') + 1;
        continue;
      }

      const spacing = /^\\[a-zA-Z]+\s*/.exec(rest);
      if (spacing) {
        const command = spacing[0].trim();
        if (['\\par', '\\newline', '\\noindent', '\\hfill'].includes(command)) {
          output += command === '\\par' ? '<br /><br />' : '<br />';
          index += spacing[0].length;
          continue;
        }
        if (symbols.has(command)) {
          try {
            output += await renderMath(command, false);
          } catch {
            output += escapeHtml(command);
          }
        }
        index += spacing[0].length;
        continue;
      }

      const character = source[index];
      if (character === '\n') output += '<br />';
      else if (character === '~') output += '&nbsp;';
      else output += escapeHtml(character);
      index += 1;
    }

    return output;
  }

  $effect(() => {
    let active = true;
    const source = text;
    html = '';
    build(source)
      .then((output) => {
        if (active) html = output;
      })
      .catch(() => {
        if (active) html = escapeHtml(source);
      });
    return () => {
      active = false;
    };
  });
</script>

<span class="latex-text"
  >{#if html}{@html html}{:else}{text}{/if}</span
>

<style>
  .latex-text :global(.katex) {
    font-size: 1em;
  }

  .latex-text :global(code) {
    font-family: var(--mono);
    font-size: 0.92em;
  }
</style>
