import React, { useMemo, useRef, useEffect, useState, useCallback } from 'react';
import ForceGraph3D from 'react-force-graph-3d';
// @ts-ignore
import ForceGraph2D from 'react-force-graph-2d';
import {
  FloatButton,
  Input,
  Modal,
  Segmented,
  Dropdown,
  Slider,
  Button,
  Card,
  Space,
  Tag,
  Typography,
  Radio,
  Result,
  Select,
} from 'antd';
import {
  CloseOutlined,
  AimOutlined,
  RedoOutlined,
  AppstoreOutlined,
  SearchOutlined,
  ReloadOutlined,
  DeploymentUnitOutlined,
} from '@ant-design/icons';
import * as THREE from 'three';
import { useTheme } from '@/context/ThemeContext';
import { scaleColor } from '@/utils/color';
import { getPlagiarismGraph, listMossReports } from '@/services/modules/assignments/plagiarism/get';
import type { MossReport, PlagiarismGraphLink } from '@/types/modules/assignments/plagiarism';

// ---------- small tips ----------
const Tips: React.FC = () => (
  <Typography.Text type="secondary" style={{ fontSize: 12 }}>
    <span className="opacity-70">
      <kbd>/</kbd> adjust filters • <kbd>F</kbd> fit • <kbd>R</kbd> reheat
    </span>
  </Typography.Text>
);

export type PlagiarismGraphProps = {
  open: boolean;
  onClose: () => void;
  moduleId: number;
  assignmentId: number;
  title?: string;
};

type Mode = '2d' | '3d';

const LABEL_2D_PX = 16; // keep as your knob
const LABEL_3D_PX = 22; // your screen-fixed height in px
const FIT_PAD_PX_2D = 8;
const FIT_PAD_PX_3D = 12;

// ---------- helpers ----------
const clamp01 = (x: number) => Math.max(0, Math.min(1, x));
const opacityForSim = (v: unknown) => {
  const s = clamp01((Number(v) || 0) / 100);
  return s < 0.2 ? 0.1 : 0.25 + (s - 0.2) * (0.75 / 0.8);
};
const withAlpha = (rgb: string, a: number) =>
  rgb.trim().startsWith('rgb(')
    ? rgb.replace(/^rgb\(([^)]+)\)$/, (_: any, inner: string) => `rgba(${inner}, ${a})`)
    : rgb;
const getSim = (l: { similarity?: number }) =>
  Math.max(0, Math.min(100, Number(l.similarity ?? 0)));

const nodeIdFrom = (username: string, reportId?: number | null) =>
  `${username}::${reportId ?? 'no_report'}`;

const usernameFromNodeId = (id: string) => (id.includes('::') ? id.split('::')[0] : id);

// cached sprite maker for 3D text
const spriteCache = new Map<string, THREE.Sprite>();
function makeTextSprite(
  key: string,
  text: string,
  opts: { bg: string; fg: string; pad?: number; fontSize?: number } = {
    bg: 'rgba(0,0,0,0.6)',
    fg: '#fff',
  },
) {
  const pad = opts.pad ?? 24;
  const fontSize = opts.fontSize ?? 64;
  const cacheKey = `${key}|${text}|${opts.bg}|${opts.fg}|${fontSize}`;
  const cached = spriteCache.get(cacheKey);
  if (cached) return cached.clone();

  const canvas = document.createElement('canvas');
  const ctx = canvas.getContext('2d')!;
  const font = `${fontSize}px Inter, ui-sans-serif, -apple-system, BlinkMacSystemFont, "Segoe UI"`;
  ctx.font = font;
  const m = ctx.measureText(text);
  const ascent = (m as any).actualBoundingBoxAscent ?? fontSize * 0.8;
  const descent = (m as any).actualBoundingBoxDescent ?? fontSize * 0.2;

  canvas.width = Math.ceil(m.width + pad * 2);
  canvas.height = Math.ceil(ascent + descent + pad * 2);

  const ctx2 = canvas.getContext('2d')!;
  ctx2.font = font;
  ctx2.fillStyle = opts.bg;
  ctx2.fillRect(0, 0, canvas.width, canvas.height);
  ctx2.fillStyle = opts.fg;
  ctx2.textBaseline = 'middle';
  ctx2.fillText(text, pad, canvas.height / 2);

  const texture = new THREE.CanvasTexture(canvas);
  texture.minFilter = THREE.LinearFilter;

  const material = new THREE.SpriteMaterial({ map: texture, transparent: true });
  material.depthTest = false; // keep labels visible on top
  material.depthWrite = false;

  const sprite = new THREE.Sprite(material);
  sprite.userData.isLabelSprite = true;
  sprite.userData.labelAspect = canvas.width / canvas.height;

  // give it a visible initial size (will be overridden each frame)
  sprite.scale.set(3 * sprite.userData.labelAspect, 3, 1);

  spriteCache.set(cacheKey, sprite.clone());
  return sprite;
}

// ---------- component ----------
const PlagiarismGraph: React.FC<PlagiarismGraphProps> = ({
  open,
  onClose,
  moduleId,
  assignmentId,
  title,
}) => {
  const { isDarkMode } = useTheme();

  // mode/refs
  const [mode, setMode] = useState<Mode>('2d');
  const fg2dRef = useRef<any>(null);
  const fg3dRef = useRef<any>(null);
  const shouldAutoFit = useRef(false);

  // filters
  const [status, setStatus] = useState<'all' | 'review' | 'flagged' | 'reviewed'>('all');
  const [simRange, setSimRange] = useState<[number, number]>([0, 100]);
  const [username, setUsername] = useState('');
  const [reportId, setReportId] = useState<number | 'all'>('all');

  // reports list
  const [reports, setReports] = useState<MossReport[]>([]);
  const [reportsLoading, setReportsLoading] = useState(false);

  // state
  const [panelOpen, setPanelOpen] = useState(true);
  const [loading, setLoading] = useState(false);
  const [rawLinks, setRawLinks] = useState<PlagiarismGraphLink[]>([]);
  const [error, setError] = useState<string | null>(null);

  // derived graph
  const normLinks = useMemo<PlagiarismGraphLink[]>(
    () =>
      rawLinks.map((l) => ({
        ...l,
        // make node ids unique per report, keep all other fields
        source: nodeIdFrom(String(l.source), l.report_id),
        target: nodeIdFrom(String(l.target), l.report_id),
        // defensively coerce numerics
        similarity: Number(l.similarity) || 0,
        lines_matched: Number(l.lines_matched) || 0,
      })),
    [rawLinks],
  );

  const nodes = useMemo(() => {
    const ids = new Set<string>();
    normLinks.forEach((l) => {
      ids.add(String(l.source));
      ids.add(String(l.target));
    });
    return Array.from(ids).map((id) => ({ id }));
  }, [normLinks]);

  // node selection filter (client-only)
  const [filteredNodes, setFilteredNodes] = useState(nodes);
  const [filteredLinks, setFilteredLinks] = useState(normLinks);
  const applyNodeFilter = useCallback(
    (nodeId: string | null) => {
      if (!nodeId) {
        setFilteredLinks(normLinks);
        setFilteredNodes(nodes);
        return;
      }
      const nset = new Set<string>([nodeId]);
      const kept: PlagiarismGraphLink[] = [];
      normLinks.forEach((l) => {
        const a = String(l.source),
          b = String(l.target);
        if (a === nodeId || b === nodeId) {
          kept.push(l);
          nset.add(a);
          nset.add(b);
        }
      });
      setFilteredLinks(kept);
      setFilteredNodes(Array.from(nset).map((id) => ({ id })));
    },
    [normLinks, nodes],
  );
  useEffect(() => {
    setFilteredNodes(nodes);
    setFilteredLinks(normLinks);
  }, [nodes, normLinks]);

  // fetch reports when opened / assignment changes
  useEffect(() => {
    if (!open) return;
    (async () => {
      setReportsLoading(true);
      try {
        const res = await listMossReports(moduleId, assignmentId);
        setReports(res.success ? (res.data?.reports ?? []) : []);
      } catch {
        setReports([]);
      } finally {
        setReportsLoading(false);
      }
    })();
  }, [open, moduleId, assignmentId]);

  useEffect(() => {
    if (!open || mode !== '3d') return;

    let raf = 0;
    const tmp = new THREE.Vector3();

    const tick = () => {
      const fg = fg3dRef.current;
      if (fg) {
        const cam = fg.camera?.() as THREE.PerspectiveCamera | undefined;
        const renderer = fg.renderer?.();
        const scene = fg.scene?.();

        if (cam && (cam as any).isPerspectiveCamera && renderer && scene) {
          const viewportHpx = renderer.domElement.clientHeight;
          const vFovRad = THREE.MathUtils.degToRad(cam.fov);

          scene.traverse((obj: any) => {
            if (!obj?.isSprite || !obj.userData?.isLabelSprite) return;

            obj.getWorldPosition(tmp);
            const dist = cam.position.distanceTo(tmp);
            const worldViewportH = 2 * Math.tan(vFovRad / 2) * dist;
            const desiredWorldH = (LABEL_3D_PX / viewportHpx) * worldViewportH;

            const aspect = obj.userData.labelAspect || 2;
            obj.scale.set(desiredWorldH * aspect, desiredWorldH, 1);
          });
        }
      }
      raf = requestAnimationFrame(tick);
    };

    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [open, mode]);

  // fetch graph
  const fetchGraph = useCallback(
    async (overrides?: { user?: string }) => {
      setLoading(true);
      setError(null);
      try {
        const params: any = {};
        if (status !== 'all') params.status = status;
        params.min_similarity = simRange[0];
        params.max_similarity = simRange[1];
        if (reportId !== 'all') params.report_id = reportId;
        const userParam = overrides?.user ?? username.trim();
        if (userParam) params.user = userParam;

        const res = await getPlagiarismGraph(moduleId, assignmentId, params);
        if (res.success) {
          setRawLinks((res.data?.links ?? []) as unknown as PlagiarismGraphLink[]);
          shouldAutoFit.current = true;
          setPanelOpen(false);
          applyNodeFilter(null);
        } else {
          setRawLinks([]);
          setError(res.message || 'Failed to load graph');
        }
      } catch {
        setRawLinks([]);
        setError('Failed to load graph');
      } finally {
        setLoading(false);
      }
    },
    [moduleId, assignmentId, status, simRange, username, reportId, applyNodeFilter],
  );

  // canvas size
  const [size, setSize] = useState({ w: 0, h: 0 });
  useEffect(() => {
    const handleResize = () => setSize({ w: window.innerWidth, h: window.innerHeight });
    handleResize();
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  // precompute degree (on the *currently displayed* graph)
  const deg = useMemo(() => {
    const m = new Map<string, number>();
    filteredLinks.forEach((l) => {
      const a = String(l.source),
        b = String(l.target);
      m.set(a, (m.get(a) || 0) + 1);
      m.set(b, (m.get(b) || 0) + 1);
    });
    return m;
  }, [filteredLinks]);

  // zoom to fit: ignore isolates (degree 0) if there are any edges
  const handleZoomToFit = useCallback(() => {
    const fg = mode === '2d' ? fg2dRef.current : fg3dRef.current;
    if (!fg) return;

    const pad = mode === '2d' ? FIT_PAD_PX_2D : FIT_PAD_PX_3D;
    const hasEdges = filteredLinks.length > 0;

    fg.zoomToFit(500, pad, hasEdges ? (n: any) => (deg.get(String(n.id)) || 0) > 0 : undefined);
  }, [mode, filteredLinks, deg]);

  const handleReheat = useCallback(() => {
    (mode === '2d' ? fg2dRef.current : fg3dRef.current)?.d3ReheatSimulation?.();
  }, [mode]);
  useEffect(() => {
    if (!open || panelOpen) return;
    requestAnimationFrame(() => setTimeout(handleZoomToFit, 120));
  }, [open, filteredNodes, filteredLinks, mode, handleZoomToFit, panelOpen]);

  // hotkeys
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      const k = e.key.toLowerCase();
      if (k === 'escape') {
        panelOpen ? setPanelOpen(false) : onClose();
        return;
      }
      const ae = document.activeElement as HTMLElement | null;
      if (
        ae &&
        (ae.tagName === 'INPUT' || ae.tagName === 'TEXTAREA' || (ae as any).isContentEditable)
      )
        return;
      if (k === 'f') handleZoomToFit();
      if (k === 'r') handleReheat();
      if (k === '2') setMode('2d');
      if (k === '3') setMode('3d');
      if (k === '/') setPanelOpen((v) => !v);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open, panelOpen, onClose, handleZoomToFit, handleReheat]);

  // styling
  const bg = isDarkMode ? '#101010' : '#fafafa';
  const linkColor2D = (l: any) => {
    const s = (l.status ?? l._orig?.status)?.toString().toLowerCase();
    const a = opacityForSim(getSim(l));
    return withAlpha(s === 'reviewed' ? 'rgb(34, 197, 94)' : scaleColor(getSim(l), 'gray-red'), a);
  };
  const materialCache = useMemo(() => new Map<string, THREE.LineBasicMaterial>(), []);
  const linkMaterial3D = (l: any) => {
    const s = (l.status ?? l._orig?.status)?.toString().toLowerCase();
    const color = s === 'reviewed' ? 'rgb(34, 197, 94)' : scaleColor(getSim(l), 'gray-red');
    const opacity = opacityForSim(getSim(l));
    const key = `${color}-${opacity.toFixed(2)}`;
    let mat = materialCache.get(key);
    if (!mat) {
      mat = new THREE.LineBasicMaterial({ color, transparent: opacity < 1, opacity });
      materialCache.set(key, mat);
    }
    return mat;
  };
  const linkWidth = (l: any) => {
    const sim = getSim(l);
    const base2D = isDarkMode ? 0.8 : 1.0;
    const base3D = isDarkMode ? 0.25 : 0.3;
    return mode === '2d'
      ? base2D + (sim / 100) * (5 - base2D)
      : base3D + (sim / 100) * (0.5 - base3D);
  };

  // tooltips
  const nodeLabel = useCallback((_n: any) => '', []); // text already rendered on node
  const linkLabel = useCallback((l: PlagiarismGraphLink) => {
    const lines = Number(l.lines_matched) || 0;
    const rep = l.report_id ?? '—';
    return `${getSim(l)}% • ${lines} lines • report ${rep}`;
  }, []);

  // context menu
  const [contextNode, setContextNode] = useState<any | null>(null);
  const [menuPos, setMenuPos] = useState<{ x: number; y: number } | null>(null);
  const onNodeRightClick = (node: any, e: MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setContextNode(node);
    setMenuPos({ x: e.clientX, y: e.clientY });
  };
  const clearAll = useCallback(() => {
    setStatus('all');
    setSimRange([0, 100]);
    setUsername('');
    setReportId('all');
    setRawLinks([]);
    setError(null);
    applyNodeFilter(null);
    setPanelOpen(true);
  }, [applyNodeFilter]);
  const clearUsernameFilter = useCallback(() => {
    setUsername('');
    fetchGraph({ user: '' });
  }, [fetchGraph]);

  const nodeCanvasObject = useCallback(
    (node: any, ctx: CanvasRenderingContext2D, globalScale: number) => {
      const label = usernameFromNodeId(String(node.id));
      const pad = 4; // px
      const inv = 1 / (globalScale || 1);

      ctx.save();
      ctx.translate(node.x, node.y);
      ctx.scale(inv, inv); // <<< key: cancel zoom
      ctx.font = `${LABEL_2D_PX}px Inter, ui-sans-serif, -apple-system, BlinkMacSystemFont, "Segoe UI"`;

      const { width } = ctx.measureText(label);
      const w = width + pad * 2;
      const h = LABEL_2D_PX + pad * 2;

      ctx.fillStyle = isDarkMode ? 'rgba(0,0,0,0.6)' : 'rgba(255,255,255,0.9)';
      ctx.fillRect(-w / 2, -h / 2, w, h);

      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.fillStyle = isDarkMode ? '#e5e7eb' : '#111827';
      ctx.fillText(label, 0, 0);

      ctx.restore();
    },
    [isDarkMode],
  );

  const nodePointerAreaPaint = useCallback(
    (node: any, color: string, ctx: CanvasRenderingContext2D, globalScale: number) => {
      const label = usernameFromNodeId(String(node.id));
      const pad = 4;
      const inv = 1 / (globalScale || 1);

      ctx.save();
      ctx.translate(node.x, node.y);
      ctx.scale(inv, inv);
      ctx.font = `${LABEL_2D_PX}px Inter, ui-sans-serif, -apple-system, BlinkMacSystemFont, "Segoe UI"`;
      const { width } = ctx.measureText(label);
      const w = width + pad * 2;
      const h = LABEL_2D_PX + pad * 2;

      ctx.fillStyle = color;
      ctx.fillRect(-w / 2, -h / 2, w, h);
      ctx.restore();
    },
    [],
  );

  // 3D custom node object: ALWAYS text sprite
  const nodeThreeObject = useCallback(
    (node: any) =>
      makeTextSprite(`txt|${isDarkMode ? 'd' : 'l'}`, usernameFromNodeId(String(node.id)), {
        bg: isDarkMode ? 'rgba(0,0,0,0.6)' : 'rgba(255,255,255,0.92)',
        fg: isDarkMode ? '#e5e7eb' : '#111827',
      }),
    [isDarkMode],
  );

  // --------- UI ----------
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
          onChange={(v) => setMode(v === '2D' ? '2d' : '3d')}
          options={[
            { label: '2D', value: '2D' },
            { label: '3D', value: '3D' },
          ]}
        />
      </div>

      {/* Graph / Panel */}
      <div className="absolute top-12 left-0 right-0 bottom-0">
        {!panelOpen ? (
          <>
            {mode === '2d' ? (
              <ForceGraph2D
                ref={fg2dRef}
                graphData={{ nodes: filteredNodes, links: filteredLinks }}
                backgroundColor={bg}
                nodeLabel={nodeLabel}
                linkLabel={linkLabel}
                nodeCanvasObjectMode={() => 'replace'}
                nodeCanvasObject={nodeCanvasObject}
                nodePointerAreaPaint={nodePointerAreaPaint}
                linkDirectionalArrowLength={0}
                linkWidth={linkWidth}
                linkColor={linkColor2D}
                linkHoverPrecision={4}
                width={size.w}
                height={Math.max(0, size.h - 48)}
                cooldownTicks={90}
                onNodeRightClick={onNodeRightClick}
                onEngineStop={() => {
                  if (shouldAutoFit.current) {
                    shouldAutoFit.current = false;
                    handleZoomToFit();
                  }
                }}
              />
            ) : (
              <ForceGraph3D
                ref={fg3dRef}
                graphData={{ nodes: filteredNodes, links: filteredLinks }}
                backgroundColor={bg}
                nodeLabel={nodeLabel}
                linkLabel={linkLabel}
                nodeThreeObject={nodeThreeObject}
                linkDirectionalArrowLength={0}
                linkWidth={linkWidth}
                linkMaterial={linkMaterial3D}
                linkHoverPrecision={4}
                width={size.w}
                height={Math.max(0, size.h - 48)}
                cooldownTicks={90}
                enableNavigationControls
                onNodeRightClick={onNodeRightClick}
                onEngineStop={() => {
                  if (shouldAutoFit.current) {
                    shouldAutoFit.current = false;
                    handleZoomToFit();
                  }
                }}
              />
            )}

            {/* Empty-state */}
            {!loading && (rawLinks.length === 0 || filteredLinks.length === 0) && (
              <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
                <Card className="pointer-events-auto shadow-lg">
                  <Result
                    status="info"
                    title={
                      rawLinks.length === 0
                        ? 'No connections found'
                        : 'Nothing matches your current view'
                    }
                    subTitle={
                      rawLinks.length === 0
                        ? 'Try adjusting filters to find connections.'
                        : 'Your current focus/filters hide everything. Clear them or adjust to see more.'
                    }
                    extra={
                      <Space className="w-full justify-between items-center">
                        <Tips />
                        <Space>
                          <Button onClick={() => setPanelOpen(true)}>Adjust filters</Button>
                          <Button onClick={clearAll}>Clear all</Button>
                        </Space>
                      </Space>
                    }
                  />
                </Card>
              </div>
            )}
          </>
        ) : (
          <div className="w-full h-full flex items-center justify-center bg-gray-50 dark:bg-gray-950">
            <Card
              className="w-[820px] max-w-[96vw] shadow-lg"
              title="Build plagiarism graph"
              extra={<Tag color="blue">{mode.toUpperCase()}</Tag>}
            >
              <Space direction="vertical" size="large" className="w-full">
                <Space className="w-full" size="large" wrap>
                  <div className="flex-1 min-w-[220px]">
                    <Typography.Text type="secondary">Status</Typography.Text>
                    <div className="mt-2">
                      <Radio.Group
                        value={status}
                        onChange={(e) => setStatus(e.target.value)}
                        optionType="button"
                        buttonStyle="solid"
                      >
                        <Radio.Button value="all">All</Radio.Button>
                        <Radio.Button value="review">Review</Radio.Button>
                        <Radio.Button value="flagged">Flagged</Radio.Button>
                        <Radio.Button value="reviewed">Reviewed</Radio.Button>
                      </Radio.Group>
                    </div>
                  </div>

                  <div className="flex-1 min-w-[260px]">
                    <Typography.Text type="secondary">
                      Similarity range: {simRange[0]}% - {simRange[1]}%
                    </Typography.Text>
                    <div className="mt-2 px-1">
                      <Slider
                        range
                        min={0}
                        max={100}
                        value={simRange}
                        onChange={(v) => Array.isArray(v) && setSimRange([v[0], v[1]])}
                        marks={{ 0: '0', 20: '20', 40: '40', 60: '60', 80: '80', 100: '100' }}
                        tooltip={{ formatter: (v) => `${v}%` }}
                      />
                    </div>
                  </div>

                  <div className="flex-1 min-w-[220px]">
                    <Typography.Text type="secondary">Filter by username</Typography.Text>
                    <Input
                      placeholder="partial match (e.g. u1234)"
                      allowClear
                      value={username}
                      onChange={(e) => setUsername(e.target.value)}
                      onPressEnter={() => fetchGraph()}
                      className="mt-2"
                    />
                  </div>

                  <div className="flex-1 min-w-[260px]">
                    <Typography.Text type="secondary">Report</Typography.Text>
                    <Select
                      className="mt-2 w-full"
                      loading={reportsLoading}
                      value={reportId}
                      onChange={(v) => setReportId(v)}
                      options={[
                        { label: 'All reports', value: 'all' as const },
                        ...reports.map((r) => ({
                          label: `#${r.id} — ${new Date(r.generated_at).toLocaleString()}`,
                          value: r.id,
                        })),
                      ]}
                    />
                  </div>
                </Space>

                {error && <Typography.Text type="danger">Error: {error}</Typography.Text>}

                <Space className="w-full justify-between items-center">
                  <Tips />
                  <Space>
                    <Button
                      icon={<ReloadOutlined />}
                      onClick={() => {
                        setStatus('all');
                        setSimRange([0, 100]);
                        setUsername('');
                        setReportId('all');
                      }}
                    >
                      Reset fields
                    </Button>
                    <Button
                      type="primary"
                      loading={loading}
                      onClick={() => fetchGraph()}
                      icon={<DeploymentUnitOutlined />}
                    >
                      Load graph
                    </Button>
                  </Space>
                </Space>
              </Space>
            </Card>
          </div>
        )}
      </div>

      {/* Context Dropdown */}
      {contextNode && menuPos && (
        <>
          <div
            onClick={() => {
              setContextNode(null);
              setMenuPos(null);
            }}
            style={{ position: 'fixed', inset: 0, zIndex: 1999 }}
          />
          <Dropdown
            menu={{
              items: [
                {
                  key: 'filter',
                  label: 'Show connections',
                  onClick: () => {
                    if (contextNode) {
                      const nid = String(contextNode.id);
                      const uname = usernameFromNodeId(nid); // extract username
                      setUsername(uname);
                      setPanelOpen(false);
                      fetchGraph({ user: uname }); // backend expects username substring
                    }
                    setContextNode(null);
                    setMenuPos(null);
                  },
                },
                {
                  key: 'clear-user',
                  label: 'Clear username filter',
                  onClick: () => {
                    clearUsernameFilter();
                    setContextNode(null);
                    setMenuPos(null);
                  },
                },
              ],
            }}
            open
            trigger={[]}
            onOpenChange={(o) => {
              if (!o) {
                setContextNode(null);
                setMenuPos(null);
              }
            }}
          >
            <div
              style={{
                position: 'fixed',
                top: menuPos.y,
                left: menuPos.x,
                zIndex: 2000,
                width: 1,
                height: 1,
              }}
            />
          </Dropdown>
        </>
      )}

      {/* Floating Actions */}
      <FloatButton
        shape="circle"
        icon={<CloseOutlined />}
        type="primary"
        tooltip="Close (Esc)"
        onClick={onClose}
        style={{ right: 24, bottom: 24, position: 'fixed', zIndex: 1001 }}
      />

      <FloatButton.Group
        shape="circle"
        trigger="click"
        style={{ right: 24, bottom: 88, position: 'fixed', zIndex: 1001 }}
        icon={<AppstoreOutlined />}
      >
        <FloatButton icon={<AimOutlined />} tooltip="Zoom to fit (F)" onClick={handleZoomToFit} />
        <FloatButton icon={<RedoOutlined />} tooltip="Reheat (R)" onClick={handleReheat} />
        <FloatButton
          icon={<SearchOutlined />}
          tooltip="Adjust filters (/)"
          onClick={() => setPanelOpen((v) => !v)}
        />
      </FloatButton.Group>

      {/* Quick search modal disabled */}
      <Modal title="Quick Search Node" open={false} onOk={() => {}} onCancel={() => {}} />
    </div>
  );
};

export default PlagiarismGraph;
