import { useEffect, useState, useRef, useCallback, useMemo } from 'react';
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
    const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
    const fgRef = useRef<any>(null);
    const inputRef = useRef<HTMLInputElement>(null);
    const containerRef = useRef<HTMLDivElement>(null);

    const [showSuggestions, setShowSuggestions] = useState(false);

    const suggestions = useMemo(() => {
        if (!searchValue) return [];
        const lowerSearch = searchValue.toLowerCase();
        return graphData.nodes
            .map(node => node.id)
            .filter(id => id.toLowerCase().startsWith(lowerSearch))
            .slice(0, 20);
    }, [searchValue, graphData.nodes]);

    useEffect(() => {
        setShowSuggestions(searchValue.length > 0 && suggestions.length > 0);
    }, [searchValue, suggestions]);

    useEffect(() => {
        fetch('/links.json')
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

    useEffect(() => {
        function handleClickOutside(event: MouseEvent) {
            if (
                containerRef.current &&
                !containerRef.current.contains(event.target as Node)
            ) {
                setShowSuggestions(false);
            }
        }
        document.addEventListener('mousedown', handleClickOutside);
        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
        };
    }, []);

    const focusNode = useCallback((nodeId: string) => {
        const node = graphData.nodes.find((n) => n.id === nodeId);
        if (node && fgRef.current && node.x != null && node.y != null && node.z != null) {
            const distance = 40;
            const distRatio = 1 + distance / Math.hypot(node.x, node.y, node.z);

            fgRef.current.cameraPosition(
                { x: node.x * distRatio, y: node.y * distRatio, z: node.z * distRatio },
                node,
                3000
            );
        }
    }, [graphData.nodes]);

    const handleSearch = useCallback(() => {
        if (selectedNodeId) {
            focusNode(selectedNodeId);
        } else if (searchValue) {
            focusNode(searchValue);
        }
    }, [focusNode, searchValue, selectedNodeId]);

    const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.key === 'Enter') {
            e.preventDefault();
            handleSearch();
            setShowSuggestions(false);
        }
    };

    const handleSuggestionClick = (id: string) => {
        setSearchValue(id);
        setSelectedNodeId(id);
        setShowSuggestions(false);
        focusNode(id);
    };

    return (
        <AppLayout title="Plagiarism View">
            <div
                ref={containerRef}
                style={{ position: 'relative', padding: '1rem', maxWidth: 100 }}
            >
                <div style={{ display: 'flex', gap: '1rem', alignItems: 'center' }}>
                    <input
                        ref={inputRef}
                        type="text"
                        placeholder="Search by node ID"
                        value={searchValue}
                        onChange={(e) => {
                            setSearchValue(e.target.value);
                            setSelectedNodeId(null);
                        }}
                        onKeyDown={handleKeyDown}
                        style={{
                            padding: '0.5rem',
                            fontSize: '1rem',
                            flexGrow: 1,
                            border: '1px solid #ccc',
                            borderRadius: '4px',
                            outline: 'none',
                            boxSizing: 'border-box',
                        }}

                        autoComplete="off"
                    />
                </div>
                {showSuggestions && (
                    <ul
                        style={{
                            border: '1px solid #ccc',
                            borderRadius: 4,
                            maxHeight: 150,
                            overflowY: 'auto',
                            margin: 0,
                            padding: '0.25rem 0',
                            listStyle: 'none',
                            backgroundColor: 'white',
                            boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
                            cursor: 'pointer',
                            zIndex: 1000,
                            position: 'absolute',
                            width: inputRef.current?.offsetWidth ?? 'auto',
                        }}
                    >
                        {suggestions.map((id) => (
                            <li
                                key={id}
                                onClick={() => handleSuggestionClick(id)}
                                style={{
                                    padding: '0.5rem 1rem',
                                    backgroundColor: id === selectedNodeId ? '#ddd' : 'transparent',
                                }}
                                onMouseDown={(e) => e.preventDefault()}
                            >
                                {id}
                            </li>
                        ))}
                    </ul>
                )}
            </div>

            <div>
                <ForceGraph3D
                    ref={fgRef}
                    graphData={graphData}
                    nodeColor={(node: GraphNode) => {
                        const maxLinks = Math.max(...graphData.nodes.map(n => n.linkCount));
                        const ratio = Math.min(node.linkCount, maxLinks) / maxLinks;
                        const hue = (1 - ratio) * 120;
                        return `hsl(${hue}, 100%, 50%)`;
                    }}
                    nodeOpacity={1}
                    nodeLabel={(node: GraphNode) => `${node.id} (${node.linkCount} links)`}
                    linkLabel={(link: GraphLink) => `${link.count} link${link.count && link.count > 1 ? 's' : ''}`}
                    backgroundColor="#ffffff"
                    linkColor={() => '#000000'}
                    enableNodeDrag={false}
                    nodeResolution={20}
                    warmupTicks={50}
                    cooldownTicks={200}
                    onEngineStop={() => fgRef.current?.zoomToFit(50)}
                    showNavInfo={false}
                    controlType='fly'
                />
            </div>
        </AppLayout>
    );
}
