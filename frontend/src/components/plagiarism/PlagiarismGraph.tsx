// components/plagiarism/PlagiarismGraph.tsx
import React, { useMemo, useRef, useEffect, useState, useCallback } from 'react';
import ForceGraph3D from 'react-force-graph-3d';
// @ts-ignore
import ForceGraph2D from 'react-force-graph-2d';
import { FloatButton, Segmented } from 'antd';
import { CloseOutlined, AimOutlined, RedoOutlined, AppstoreOutlined } from '@ant-design/icons';
import { useTheme } from '@/context/ThemeContext';

export type PlagiarismGraphProps = {
  open: boolean;
  onClose: () => void;
  links: { source: any; target: any }[];
  title?: string;
};

type Mode = '2d' | '3d';

const idOf = (v: any) => {
  if (v == null) return '';
  if (typeof v === 'string' || typeof v === 'number') return String(v);
  if ((v as any).id != null) return String((v as any).id);
  if ((v as any).username != null) return String((v as any).username);
  if ((v as any).value != null) return String((v as any).value);
  try {
    return JSON.stringify(v);
  } catch {
    return String(v);
  }
};

const PlagiarismGraph: React.FC<PlagiarismGraphProps> = ({ open, onClose, links, title }) => {
  const { isDarkMode } = useTheme();

  // --- Mode & refs (do not early-return before hooks)
  const [mode, setMode] = useState<Mode>('2d');
  const fg2dRef = useRef<any>(null);
  const fg3dRef = useRef<any>(null);

  // --- Normalize links (keep original ref if ever needed)
  const normLinks = useMemo(
    () => links.map((l) => ({ source: idOf(l.source), target: idOf(l.target), _orig: l })),
    [links],
  );

  // --- Nodes
  const nodes = useMemo(() => {
    const ids = new Set<string>();
    normLinks.forEach((l) => {
      ids.add(l.source as string);
      ids.add(l.target as string);
    });
    return Array.from(ids).map((id) => ({ id }));
  }, [normLinks]);

  // --- Adjacency & per-node original link set (for highlight)
  const { neighborsById, linksById } = useMemo(() => {
    const nMap = new Map<string, Set<string>>();
    const lMap = new Map<string, Set<any>>();
    nodes.forEach((n) => {
      nMap.set(n.id, new Set());
      lMap.set(n.id, new Set());
    });
    normLinks.forEach((l) => {
      const a = String(l.source);
      const b = String(l.target);
      nMap.get(a)?.add(b);
      nMap.get(b)?.add(a);
      lMap.get(a)?.add(l._orig);
      lMap.get(b)?.add(l._orig);
    });
    return { neighborsById: nMap, linksById: lMap };
  }, [nodes, normLinks]);

  // --- Highlight state
  const [highlightNodes, setHighlightNodes] = useState<Set<string>>(new Set());
  const [highlightLinks, setHighlightLinks] = useState<Set<any>>(new Set());
  const [hoverNodeId, setHoverNodeId] = useState<string | null>(null);
  const hoverActive = highlightNodes.size > 0 || highlightLinks.size > 0;

  const clearHighlight = useCallback(() => {
    setHighlightNodes(new Set());
    setHighlightLinks(new Set());
    setHoverNodeId(null);
  }, []);

  const handleNodeHover = useCallback(
    (node: any | null) => {
      if (!node) {
        clearHighlight();
        return;
      }
      const nid = String(node.id);
      const nSet = new Set<string>();
      const lSet = new Set<any>();
      nSet.add(nid);
      neighborsById.get(nid)?.forEach((nbr) => nSet.add(nbr));
      linksById.get(nid)?.forEach((lnk) => lSet.add(lnk));
      setHighlightNodes(nSet);
      setHighlightLinks(lSet);
      setHoverNodeId(nid);
    },
    [clearHighlight, neighborsById, linksById],
  );

  const handleLinkHover = useCallback(
    (lShown: any | null) => {
      if (!lShown) {
        clearHighlight();
        return;
      }
      const orig = lShown._orig ?? lShown;
      const src = String(orig.source?.id ?? orig.source);
      const tgt = String(orig.target?.id ?? orig.target);
      setHighlightNodes(new Set<string>([src, tgt]));
      setHighlightLinks(new Set<any>([orig]));
      setHoverNodeId(null);
    },
    [clearHighlight],
  );

  // --- Size
  const [size, setSize] = useState<{ w: number; h: number }>({
    w: typeof window !== 'undefined' ? window.innerWidth : 0,
    h: typeof window !== 'undefined' ? window.innerHeight : 0,
  });
  const handleResize = useCallback(
    () => setSize({ w: window.innerWidth, h: window.innerHeight }),
    [],
  );
  useEffect(() => {
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, [handleResize]);

  // --- Hotkeys
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
      const k = e.key.toLowerCase();
      if (k === 'f')
        mode === '2d'
          ? fg2dRef.current?.zoomToFit?.(500, 40)
          : fg3dRef.current?.zoomToFit?.(500, 60);
      if (k === 'r')
        mode === '2d'
          ? fg2dRef.current?.d3ReheatSimulation?.()
          : fg3dRef.current?.d3ReheatSimulation?.();
      if (k === '2') setMode('2d');
      if (k === '3') setMode('3d');
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open, onClose, mode]);

  // --- 2D highlight ring painter
  const NODE_R = 3;
  const HIGHLIGHT_MULT = 2.0; // adjust ring radius multiplier
  const paintRing = useCallback(
    (node: any, ctx: CanvasRenderingContext2D) => {
      if (!highlightNodes.has(String(node.id))) return;
      ctx.beginPath();
      ctx.arc(node.x, node.y, NODE_R * HIGHLIGHT_MULT, 0, 2 * Math.PI, false);
      ctx.fillStyle = hoverNodeId === String(node.id) ? '#ff3b30' : '#ff9800';
      ctx.globalAlpha = 0.25;
      ctx.fill();
      ctx.globalAlpha = 1;
    },
    [highlightNodes, hoverNodeId],
  );

  // --- Theme & predicates
  const bg = isDarkMode ? '#101010' : '#fafafa';
  const baseLink = isDarkMode ? '#737373' : '#737373';
  const hlNode = '#ff9800';
  const dimNode = isDarkMode ? '#555' : '#bbb';
  const dimLink = isDarkMode ? '#444' : '#d0d0d0';

  const isNodeHighlighted = (n: any) => highlightNodes.has(String(n.id));
  const isLinkHighlighted = (l: any) => highlightLinks.has(l._orig ?? l);

  // --- Actions
  const handleZoomToFit = () =>
    mode === '2d' ? fg2dRef.current?.zoomToFit?.(500, 40) : fg3dRef.current?.zoomToFit?.(500, 60);
  const handleReheat = () =>
    mode === '2d'
      ? fg2dRef.current?.d3ReheatSimulation?.()
      : fg3dRef.current?.d3ReheatSimulation?.();

  // --- Render (hide when closed but keep mounted to preserve hooks)
  return (
    <div
      aria-label={title ?? 'Plagiarism Graph'}
      role="dialog"
      className="fixed inset-0 z-[1000]"
      style={{ display: open ? 'block' : 'none' }}
      aria-hidden={!open}
    >
      {/* Header */}
      <div className="absolute left-0 right-0 top-0 h-12 flex items-center justify-between px-4 border-b border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950">
        <span className="font-semibold">{title ?? 'Plagiarism Graph'}</span>
        <Segmented
          size="small"
          value={mode === '2d' ? '2D' : '3D'}
          onChange={(val) => setMode(val === '2D' ? '2d' : '3d')}
          options={[
            { label: '2D', value: '2D' },
            { label: '3D', value: '3D' },
          ]}
        />
      </div>

      {/* Graph area */}
      <div className="absolute top-12 left-0 right-0 bottom-0">
        {mode === '2d' ? (
          <ForceGraph2D
            ref={fg2dRef}
            graphData={{ nodes, links: normLinks }}
            backgroundColor={bg}
            nodeAutoColorBy="id"
            nodeLabel={(n: any) => String(n.id)} // tooltip only
            linkDirectionalArrowLength={0}
            linkWidth={(l: any) => (isLinkHighlighted(l) ? 3 : 1.2)}
            linkColor={(l: any) =>
              hoverActive ? (isLinkHighlighted(l) ? hlNode : dimLink) : baseLink
            }
            nodeCanvasObjectMode={(n: any) => (isNodeHighlighted(n) ? 'before' : undefined)}
            nodeCanvasObject={(node: any, ctx: CanvasRenderingContext2D) => {
              paintRing(node, ctx);
              ctx.beginPath();
              ctx.arc(node.x, node.y, NODE_R, 0, 2 * Math.PI, false);
              const c = (node as any).color || '#8884d8';
              ctx.fillStyle = hoverActive ? (isNodeHighlighted(node) ? hlNode : dimNode) : c;
              ctx.globalAlpha = 0.9;
              ctx.fill();
              ctx.globalAlpha = 1;
            }}
            width={size.w}
            height={size.h - 48}
            cooldownTicks={90}
            onNodeHover={handleNodeHover}
            onLinkHover={(l: any | null) => handleLinkHover(l?._orig ? l : l)}
          />
        ) : (
          <ForceGraph3D
            ref={fg3dRef}
            graphData={{ nodes, links: normLinks }}
            backgroundColor={bg}
            nodeAutoColorBy="id"
            nodeColor={(n: any) =>
              hoverActive ? (isNodeHighlighted(n) ? hlNode : dimNode) : n.color || undefined
            }
            nodeOpacity={1}
            nodeLabel={(n: any) => String(n.id)}
            linkDirectionalArrowLength={0}
            linkWidth={(l: any) => (isLinkHighlighted(l) ? 2.5 : 0.4)}
            linkColor={(l: any) =>
              hoverActive ? (isLinkHighlighted(l) ? hlNode : dimLink) : baseLink
            }
            width={size.w}
            height={size.h - 48}
            cooldownTicks={90}
            enableNavigationControls
            onNodeHover={handleNodeHover}
            onLinkHover={(l: any | null) => handleLinkHover(l?._orig ? l : l)}
          />
        )}
      </div>

      {/* Close button fixed at very bottom */}
      <FloatButton
        shape="circle"
        icon={<CloseOutlined />}
        type="primary"
        tooltip="Close (Esc)"
        onClick={onClose}
        style={{ right: 24, bottom: 24, position: 'fixed', zIndex: 1001 }}
      />

      {/* Expandable menu just above the close button */}
      <FloatButton.Group
        shape="circle"
        trigger="click"
        style={{ right: 24, bottom: 88, position: 'fixed', zIndex: 1001 }} // 88 = 24 (padding) + 40 (button height) + some gap
        icon={<AppstoreOutlined />}
      >
        <FloatButton icon={<AimOutlined />} tooltip="Zoom to fit (F)" onClick={handleZoomToFit} />
        <FloatButton icon={<RedoOutlined />} tooltip="Reheat (R)" onClick={handleReheat} />
      </FloatButton.Group>
    </div>
  );
};

export default PlagiarismGraph;
