import { useEffect, useState, useRef, useCallback } from 'react';
import { useParams } from 'react-router-dom';
import AppLayout from '@/layouts/AppLayout';
import ForceGraph3D from 'react-force-graph-3d';
import { ModulesService } from '@/services/modules';
import type { ModuleDetailsResponse } from '@/types/modules';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

type GraphNode = {
    id: string;
    linkCount: number;
    x?: number;
    y?: number;
    z?: number;
};

type GraphLink = {
    source: string;
    target: string;
    count?: number;
};

type GraphData = {
    nodes: GraphNode[];
    links: GraphLink[];
};

export default function PlagiarismView() {
    const { id } = useParams<{ id: string }>();
    const moduleId = parseInt(id ?? '', 10);
    const { setBreadcrumbLabel } = useBreadcrumbContext();

    const [moduleDetails, setModuleDetails] = useState<ModuleDetailsResponse | null>(null);
    const [loading, setLoading] = useState(true);
    const [graphData, setGraphData] = useState<GraphData>({ nodes: [], links: [] });
    const [searchValue, setSearchValue] = useState('');
    const fgRef = useRef<any>(null);

    useEffect(() => {
        fetch('/plagiarismGraph.json')
            .then(res => res.json())
            .then((data: { links: GraphLink[] }) => {

                const uniqueNodeIds = new Set<string>();
                data.links.forEach(link => {
                    uniqueNodeIds.add(link.source);
                    uniqueNodeIds.add(link.target);
                });

                const linkCounts: Record<string, number> = {};
                data.links.forEach(link => {
                    linkCounts[link.source] = (linkCounts[link.source] || 0) + 1;
                    linkCounts[link.target] = (linkCounts[link.target] || 0) + 1;
                });

                const nodes: GraphNode[] = Array.from(uniqueNodeIds).map(id => ({
                    id,
                    linkCount: linkCounts[id] || 0,
                }));

                const linkCountMap: Record<string, number> = {};
                data.links.forEach(link => {
                    const key = [link.source, link.target].sort().join('||');
                    linkCountMap[key] = (linkCountMap[key] || 0) + 1;
                });

                const linksWithCount = data.links.map(link => {
                    const key = [link.source, link.target].sort().join('||');
                    return { ...link, count: linkCountMap[key] };
                });

                setGraphData({ nodes, links: linksWithCount });
            });
    }, []);

    useEffect(() => {
        ModulesService.getModuleDetails(moduleId).then((res) => {
            if (res.success) {
                setModuleDetails(res.data);
                setBreadcrumbLabel(
                    `modules/${res.data.id}`,
                    `${res.data.code} - ${res.data.description || 'Module'}`
                );
            }
            setLoading(false);
        });
    }, [moduleId, setBreadcrumbLabel]);

    useEffect(() => {
        if (!loading && fgRef.current) {
            fgRef.current.zoomToFit(20);
        }
    }, [loading]);

    const handleSearch = useCallback(() => {
        const node = graphData.nodes.find((n) => n.id === searchValue);
        if (node && fgRef.current && node.x != null && node.y != null && node.z != null) {
            const distance = 1;
            const distRatio = 1 + distance / Math.hypot(node.x, node.y, node.z);

            fgRef.current.cameraPosition(
                {
                    x: node.x * distRatio,
                    y: node.y * distRatio,
                    z: node.z * distRatio
                },
                node,
                3000
            );
        }
    }, [searchValue, graphData]);

    return (
        <AppLayout title="Plagiarism View">
            <div style={{ padding: '1rem', display: 'flex', gap: '1rem', alignItems: 'center' }}>
                <input
                    type="text"
                    placeholder="Search by node ID"
                    value={searchValue}
                    onChange={(e) => setSearchValue(e.target.value)}
                    style={{ padding: '0.5rem', fontSize: '1rem' }}
                />
                <button onClick={handleSearch} style={{ padding: '0.5rem 1rem' }}>
                    Focus
                </button>
            </div>

            <div style={{ width: '100vw', height: '80vh' }}>
                <ForceGraph3D
                    ref={fgRef}
                    graphData={graphData}
                    nodeColor={(node: GraphNode) => {
                        const maxLinks = Math.max(...graphData.nodes.map(n => n.linkCount));
                        const ratio = Math.min(node.linkCount, maxLinks) / maxLinks;
                        const hue = (1 - ratio) * 120;
                        return `hsl(${hue}, 100%, 50%)`;
                    }}
                    nodeLabel={(node: GraphNode) => `${node.id} (${node.linkCount} links)`}
                    linkLabel={(link: GraphLink) => `${link.count} link${link.count && link.count > 1 ? 's' : ''}`}
                    backgroundColor="#ffffff"
                    linkColor={() => '#000000'}
                    enableNodeDrag={false}
                />
            </div>
        </AppLayout>
    );
}
