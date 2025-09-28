import JSZip from 'jszip';

export type ArchiveEntry = {
  /** POSIX-style path inside the archive (e.g., src/main.cpp) */
  path: string;
  /** True if the entry is a directory placeholder */
  isDir: boolean;
};

export type ArchivePreviewResult = {
  type: 'zip' | 'tar' | 'unsupported';
  entries: ArchiveEntry[];
};

/**
 * List entries in a .zip file without extracting content.
 * Returns files and directory placeholders using forward-slash paths.
 */
export async function listZipEntries(input: Blob): Promise<ArchiveEntry[]> {
  const zip = await JSZip.loadAsync(input);
  const entries: ArchiveEntry[] = [];
  for (const file of Object.values(zip.files)) {
    const path = file.name.replace(/\\/g, '/');
    entries.push({ path, isDir: file.dir === true });
  }
  // Stable, natural-ish ordering: directories first, then files, both lexicographically
  entries.sort((a, b) => {
    if (a.isDir !== b.isDir) return a.isDir ? -1 : 1;
    return a.path.localeCompare(b.path);
  });
  return entries;
}

// --- Minimal TAR reader (list only) ---
function textFrom(buf: Uint8Array, start: number, len: number): string {
  let out = '';
  for (let i = 0; i < len; i++) {
    const c = buf[start + i];
    if (c === 0) break;
    out += String.fromCharCode(c);
  }
  return out.trim();
}

function parseOctal(buf: Uint8Array, start: number, len: number): number {
  const s = textFrom(buf, start, len).replace(/\0.*$/, '').trim();
  if (!s) return 0;
  return parseInt(s, 8);
}

export async function listTarEntriesFromArrayBuffer(ab: ArrayBuffer): Promise<ArchiveEntry[]> {
  const u8 = new Uint8Array(ab);
  const entries: ArchiveEntry[] = [];
  const BLOCK = 512;
  let offset = 0;
  while (offset + BLOCK <= u8.length) {
    const block = u8.subarray(offset, offset + BLOCK);
    // End of archive: two consecutive zero blocks
    const allZero = block.every((b) => b === 0);
    if (allZero) break;
    const name = textFrom(block, 0, 100);
    const size = parseOctal(block, 124, 12);
    const typeflag = block[156]; // '0' file, '5' dir, others exist
    if (name) {
      const isDir = typeflag === 53 /* '5' */ || name.endsWith('/');
      entries.push({ path: name, isDir });
    }
    // Move past header + content (content rounded up to 512)
    const contentBlocks = Math.ceil(size / BLOCK);
    offset += BLOCK + contentBlocks * BLOCK;
  }
  entries.sort((a, b) => {
    if (a.isDir !== b.isDir) return a.isDir ? -1 : 1;
    return a.path.localeCompare(b.path);
  });
  return entries;
}

async function gunzipToArrayBuffer(blob: Blob): Promise<ArrayBuffer> {
  if ('DecompressionStream' in window) {
    const ds = new (window as any).DecompressionStream('gzip');
    const stream = blob.stream().pipeThrough(ds);
    const resp = new Response(stream);
    return await resp.arrayBuffer();
  }
  throw new Error('Gzip preview not supported in this browser');
}

/**
 * Detect archive type and provide a preview listing where possible.
 * Supports: .zip; best‑effort for .tar, .tgz/.tar.gz (gzip required).
 */
export async function listArchiveEntries(file: File): Promise<ArchivePreviewResult> {
  const name = file.name.toLowerCase();
  if (name.endsWith('.zip')) {
    const entries = await listZipEntries(file);
    return { type: 'zip', entries };
  }
  if (name.endsWith('.tar')) {
    const ab = await file.arrayBuffer();
    const entries = await listTarEntriesFromArrayBuffer(ab);
    return { type: 'tar', entries };
  }
  if (name.endsWith('.tgz') || name.endsWith('.tar.gz') || name.endsWith('.gz')) {
    try {
      const ab = await gunzipToArrayBuffer(file);
      const entries = await listTarEntriesFromArrayBuffer(ab);
      return { type: 'tar', entries };
    } catch {
      return { type: 'unsupported', entries: [] };
    }
  }
  return { type: 'unsupported', entries: [] };
}
