import JSZip from 'jszip';

export type MakeTargets = string[];

const IGNORE_TARGETS = new Set([
  'all', 'clean', 'install', 'uninstall', 'dist', 'build',
  '.PHONY', 'format', 'fmt', 'lint', 'test'
]);

export function extractTargetsFromMakefile(content: string): MakeTargets {
  const lines = content.replace(/\r\n?/g, '\n').split('\n');
  const targets: string[] = [];

  for (const line of lines) {
    const m = line.match(/^([A-Za-z0-9_.-]+)\s*:(?![=]).*$/);
    if (!m) continue;
    const t = m[1];
    if (t.includes('%')) continue;
    if (IGNORE_TARGETS.has(t)) continue;
    if (!targets.includes(t)) targets.push(t);
  }

  const taskN = targets.filter((t) => /^task\d+$/i.test(t));
  return taskN.length ? taskN : targets;
}

export async function readMakefileFromZip(zipFile: File): Promise<string | null> {
  try {
    const zip = await JSZip.loadAsync(zipFile);
    const names = Object.keys(zip.files).filter((n) => !zip.files[n].dir);

    const rootMake = names.find((n) => !n.includes('/'));
    if (rootMake && /^(makefile|Makefile|MAKEFILE)(\.mk)?$/.test(rootMake)) {
      const entry = zip.file(rootMake);
      return entry ? await entry.async('string') : null;
    }

    const candidate =
      names.find((n) => /(^|\/)([Mm]akefile)(\.mk)?$/.test(n)) ||
      names.find((n) => /\.mk$/i.test(n));

    if (!candidate) return null;

    const entry = zip.file(candidate);
    return entry ? await entry.async('string') : null;
  } catch {
    return null;
  }
}

export async function parseTargetsFromMakefileZip(zipFile: File): Promise<MakeTargets> {
  const text = await readMakefileFromZip(zipFile);
  if (!text) return [];
  return extractTargetsFromMakefile(text);
}
