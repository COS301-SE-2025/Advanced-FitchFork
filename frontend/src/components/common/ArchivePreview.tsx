import React, { useEffect, useState } from 'react';
import { Alert, Tree } from 'antd';
import {
  listArchiveEntries,
  type ArchiveEntry,
  type ArchivePreviewResult,
} from '@/utils/archivePreview';

type Props = {
  file: File | null;
  /** Max tree depth to display (counting path segments). Default 5. */
  maxDepth?: number;
  className?: string;
};

const ArchivePreview: React.FC<Props> = ({ file, maxDepth = 5, className }) => {
  const [, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [entries, setEntries] = useState<ArchiveEntry[]>([]);
  const [type, setType] = useState<ArchivePreviewResult['type']>('unsupported');

  useEffect(() => {
    let cancelled = false;
    setEntries([]);
    setError(null);
    if (!file) return;
    (async () => {
      try {
        setLoading(true);
        const res = await listArchiveEntries(file);
        if (cancelled) return;
        setType(res.type);
        setEntries(res.entries);
      } catch (e: any) {
        if (!cancelled) setError(e?.message ?? 'Failed to preview archive');
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [file]);

  if (!file) return null;

  if (type === 'unsupported') {
    return (
      <Alert
        type="info"
        showIcon
        message="Preview not available for this format"
        description="Upload a .zip to see a quick listing of files inside."
        className={className}
      />
    );
  }

  if (error) {
    return (
      <Alert
        type="error"
        showIcon
        message="Could not preview archive"
        description={error}
        className={className}
      />
    );
  }

  // Filter out OS metadata files and folders
  const filtered = entries.filter((e) => {
    const p = e.path;
    if (p.includes('__MACOSX/')) return false;
    const base = p.split('/').pop() || p;
    if (base === '.DS_Store') return false;
    if (base.startsWith('._')) return false;
    return true;
  });
  // Build tree from files only to avoid duplicate explicit directory placeholders
  // Also cap by depth: include files whose path depth <= maxDepth
  const filesOnly = filtered.filter((e) => !e.isDir);
  const shown = filesOnly.filter((e) => e.path.split('/').filter(Boolean).length <= maxDepth);

  // Build a simple tree structure from paths
  type Node = { title: string; key: string; children?: Node[]; _map?: Record<string, Node> };
  const root: Record<string, Node> = {};
  const ensureNode = (parent: Record<string, Node>, seg: string, pathKey: string): Node => {
    if (!parent[seg]) parent[seg] = { title: seg, key: pathKey, children: [], _map: {} };
    // Ensure child map exists for accumulating children across multiple paths
    if (!parent[seg]._map) parent[seg]._map = {};
    return parent[seg];
  };
  const addPath = (p: string, isDir: boolean) => {
    const parts = p.split('/').filter(Boolean);
    let cursorMap = root;
    let keyPath = '';
    for (let i = 0; i < parts.length; i++) {
      const seg = parts[i];
      keyPath = keyPath ? keyPath + '/' + seg : seg;
      const node = ensureNode(cursorMap, seg, keyPath);
      if (i < parts.length - 1 || isDir) {
        // descend into children
        // descend into or create this node's child map
        cursorMap = node._map!;
      }
    }
  };
  shown.forEach((e) => addPath(e.path, false));
  // Convert map to array recursively
  const toArray = (map: Record<string, Node>): Node[] => {
    const arr = Object.values(map);
    arr.sort((a, b) => a.title.localeCompare(b.title));
    return arr.map((n) => {
      const m = n._map as Record<string, Node> | undefined;
      if (m && Object.keys(m).length) {
        return { title: n.title, key: n.key, children: toArray(m) };
      }
      return { title: n.title, key: n.key };
    });
  };
  const treeData = toArray(root);

  return (
    <div className={className}>
      <div className="text-xs text-gray-500 mb-2">Archive preview</div>
      <div className="rounded border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950 p-2">
        <Tree treeData={treeData as any} showIcon={false} selectable={false} />
      </div>
      {/* Intentionally no 
          "…and N more" line: we cap by depth and show all nodes up to that depth. */}
    </div>
  );
};

export default ArchivePreview;
