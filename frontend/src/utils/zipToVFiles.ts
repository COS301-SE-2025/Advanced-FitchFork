import JSZip from "jszip";

/** Match your IDE’s VFile shape */
export type VFile = {
  id: string;
  name: string;
  path: string[];   // folder segments
  language: string; // monaco language id
  value: string;    // file contents
};

export type ExtractOptions = {
  /** Ignore files larger than this (bytes). Default: 1 MiB */
  maxPerFileBytes?: number;
  /** Skip hidden/system files like .DS_Store and __MACOSX. Default: true */
  skipHidden?: boolean;
  /** Allow binary files as base64 strings. Default: false (skip binaries) */
  includeBinaryAsBase64?: boolean;
};

/** Simple extension → Monaco language mapping */
const extToLang = (filename: string): string => {
  const ext = filename.split(".").pop()?.toLowerCase();
  switch (ext) {
    case "ts":
    case "tsx":
      return "typescript";
    case "js":
    case "jsx":
      return "javascript";
    case "json":
      return "json";
    case "css":
      return "css";
    case "html":
      return "html";
    case "md":
      return "markdown";
    case "py":
      return "python";
    case "java":
      return "java";
    case "sql":
      return "sql";
    default:
      return "plaintext";
  }
};

const TEXT_EXTS = new Set([
  "ts","tsx","js","jsx","json","css","scss","sass","less","html","htm",
  "md","markdown","txt","xml","yml","yaml","toml","ini","env",
  "py","java","kt","kts","rs","go","rb","php","c","h","cpp","hpp",
  "cs","sql","sh","bash","zsh","fish","vue","svelte"
]);

/** Quick filename-based check first */
const looksTextyByName = (name: string) => {
  const base = name.split("/").pop() || name;
  if (base.startsWith(".")) return false; // dotfiles usually not for IDE demo
  const ext = base.split(".").pop()?.toLowerCase();
  return ext ? TEXT_EXTS.has(ext) : true; // no ext? assume text
};

/** Heuristic: treat as text if few control bytes */
const isProbablyTextBuffer = (buf: Uint8Array): boolean => {
  // Empty? call it text.
  if (buf.length === 0) return true;
  let control = 0;
  const sample = buf.subarray(0, Math.min(buf.length, 2048));
  for (let i = 0; i < sample.length; i++) {
    const b = sample[i];
    // allow \t \n \r
    if (b === 9 || b === 10 || b === 13) continue;
    // control (0..8, 11..12, 14..31) or DEL(127)
    if ((b >= 0 && b <= 8) || (b >= 11 && b <= 12) || (b >= 14 && b <= 31) || b === 127) {
      control++;
      if (control > 16) return false; // early out
    }
  }
  return true;
};

const pathToKey = (path: string[], name?: string) =>
  (name ? [...path, name] : path).join("/");

const shouldSkip = (name: string, skipHidden: boolean) => {
  if (!skipHidden) return false;
  if (name.includes("__MACOSX/")) return true;
  const base = name.split("/").pop() || name;
  if (base === ".DS_Store") return true;
  if (base.startsWith("._")) return true;
  return false;
};

/**
 * Extract a ZIP into VFiles for the IDE.
 * - Decodes text files to UTF-8 strings
 * - Skips obvious binaries by default (or includes them as base64 if requested)
 * - Preserves folder structure in `path: string[]`
 */
export async function zipToVFiles(
  input: Blob | ArrayBuffer | Uint8Array,
  opts: ExtractOptions = {}
): Promise<VFile[]> {
  const {
    maxPerFileBytes = 1024 * 1024, // 1 MiB
    skipHidden = true,
    includeBinaryAsBase64 = false,
  } = opts;

  const zip = await JSZip.loadAsync(input);
  const out: VFile[] = [];

  // Gather all non-directory entries
  const entries = Object.values(zip.files).filter(f => !f.dir);

  for (const entry of entries) {
    const fullName = entry.name.replace(/\\/g, "/"); // windows zip safety
    if (shouldSkip(fullName, skipHidden)) continue;

    // Split name into path segments
    const parts = fullName.split("/").filter(Boolean);
    if (parts.length === 0) continue;
    const name = parts.pop()!;
    const path = parts;

    // Pull bytes once
    const bytes = await entry.async("uint8array");
    if (bytes.length > maxPerFileBytes) continue;

    const textyByName = looksTextyByName(name);
    const textyByContent = textyByName || isProbablyTextBuffer(bytes);

    if (textyByContent) {
      const value = new TextDecoder("utf-8", { fatal: false }).decode(bytes);
      out.push({
        id: crypto.randomUUID(),
        name,
        path,
        language: extToLang(name),
        value,
      });
    } else if (includeBinaryAsBase64) {
      // Keep binaries as base64 so users can still export roundtrip
      const base64 = btoa(String.fromCharCode(...bytes));
      out.push({
        id: crypto.randomUUID(),
        name,
        path,
        language: "plaintext",
        value: `__BINARY_BASE64__: ${base64}`,
      });
    }
  }

  // Sort by path/name for stable tab ordering
  out.sort((a, b) => pathToKey(a.path, a.name).localeCompare(pathToKey(b.path, b.name)));
  return out;
}
