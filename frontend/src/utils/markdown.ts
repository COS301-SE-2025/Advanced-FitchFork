// Minimal Markdown → plain text (good for table previews / search indexing)
export function stripMarkdown(md: string): string {
  if (!md) return '';

  let s = md.replace(/\r\n?/g, '\n');

  // code fences (drop entirely) + inline code (keep inner)
  s = s.replace(/```[\s\S]*?```/g, '');
  s = s.replace(/`([^`]+)`/g, '$1');

  // HTML tags
  s = s.replace(/<[^>]+>/g, ' ');

  // images -> keep alt
  s = s.replace(/!\[([^\]]*)]\([^)]*\)/g, '$1');

  // links -> keep label
  s = s.replace(/\[([^\]]+)]\((?:[^()\s]+|\([^)]*\))+\)/g, '$1');

  // ref-style images/links: ![alt][id], [label][id], and definitions
  s = s.replace(/!\[([^\]]*)]\[[^\]]*]/g, '$1');
  s = s.replace(/\[([^\]]+)]\[[^\]]*]/g, '$1');
  s = s.replace(/^\s*\[[^\]]+]:\s+\S+.*$/gm, '');

  // headings, blockquotes
  s = s.replace(/^#{1,6}\s*/gm, '');
  s = s.replace(/^\s{0,3}>\s?/gm, '');

  // lists
  s = s.replace(/^\s*([-*+]|(\d+)\.)\s+/gm, '');

  // emphasis / strike
  s = s.replace(/(\*\*|__)(.*?)\1/g, '$2');
  s = s.replace(/(\*|_)(.*?)\1/g, '$2');
  s = s.replace(/~~(.*?)~~/g, '$1');

  // tables
  s = s.replace(/^\s*\|?\s*:?-{3,}:?\s*(\|\s*:?-{3,}:?\s*)+\|?\s*$/gm, '');
  s = s.replace(/\|/g, ' ');

  // footnotes
  s = s.replace(/\[\^[^\]]+](?::.*$)?/gm, '');

  // escapes
  s = s.replace(/\\([\\`*_{}\[\]()#+\-.!>~|])/g, '$1');

  // collapse whitespace
  s = s.replace(/[ \t]+\n/g, '\n');
  s = s.replace(/\n{3,}/g, '\n\n');
  s = s.replace(/\s+/g, ' ').trim();

  return s;
}

// Char-based excerpt with word-friendly cutoff
export function mdExcerpt(md: string, max = 160): string {
  const text = stripMarkdown(md);
  if (text.length <= max) return text;
  const cut = text.slice(0, max);
  const lastBreak = Math.max(cut.lastIndexOf(' '), cut.lastIndexOf('.'), cut.lastIndexOf(','));
  return ((lastBreak > max * 0.6 ? cut.slice(0, lastBreak) : cut).trimEnd()) + '…';
}
