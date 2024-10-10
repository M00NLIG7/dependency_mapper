// src/components/NetworkGraph/NetworkGraph.tsx
"use client"
import { useEffect, useRef, useState } from 'react';
import * as d3 from 'd3';
import { Button } from "@/components/ui/button";
import { Menu, Plus, Minus } from 'lucide-react';
import Sidebar from '@/components/NetworkGraphSidebar';
import GraphVisualization from '@/components/GraphVisualization';

const NetworkGraph = () => {
    const svgRef = useRef(null);
    const zoomRef = useRef(null);
    const [graphData, setGraphData] = useState({ nodes: [], links: [], connections: [] });
    const [isSidebarOpen, setIsSidebarOpen] = useState(false);
    const [selectedTypes, setSelectedTypes] = useState([]);
    const [regexFilter, setRegexFilter] = useState('');

    useEffect(() => {
        fetch('/api/graph-data')
            .then((response) => response.json())
            .then((data) => {
                const nodes = data.nodes.map((node) => ({
                    id: node.id,
                    os: node.os,
                    type: node.type,
                }));

                const connections = data.connections.map((conn) => ({
                    id: conn.id,
                    protocol: conn.protocol,
                    sourcePort: conn.sourcePort,
                    targetPort: conn.targetPort,
                    description: conn.description,
                }));

                const links = data.edges
                    .map((edge) => ({
                        source: edge.source,
                        target: edge.connection,
                    }))
                    .concat(
                        data.edges.map((edge) => ({
                            source: edge.connection,
                            target: edge.target,
                        }))
                    );

                setGraphData({ nodes, links, connections });
            });
    }, []);

    const toggleSidebar = () => {
        setIsSidebarOpen((prev) => !prev);
    };

    const handleZoom = (direction) => {
        const svg = d3.select(svgRef.current);
        const zoom = zoomRef.current;
        if (!zoom) return;

        const currentTransform = d3.zoomTransform(svg.node());
        const scale = direction === 'in' ? 1.2 : 0.8;
        const newTransform = currentTransform.scale(scale);

        svg.transition().duration(300).call(zoom.transform, newTransform);
    };

    const applyFilters = () => {
        let filteredNodes = graphData.nodes;
        if (selectedTypes.length > 0) {
            filteredNodes = filteredNodes.filter(node => selectedTypes.includes(node.type));
        }
        if (regexFilter) {
            const regex = new RegExp(regexFilter, 'i');
            filteredNodes = filteredNodes.filter(node => regex.test(node.id));
        }

        const filteredLinks = graphData.links.filter(link => 
            filteredNodes.some(node => node.id === link.source.id || node.id === link.source) &&
            filteredNodes.some(node => node.id === link.target.id || node.id === link.target)
        );
        const filteredConnections = graphData.connections.filter(conn => 
            filteredLinks.some(link => link.source === conn.id || link.target === conn.id)
        );

        setGraphData({
            nodes: filteredNodes,
            links: filteredLinks,
            connections: filteredConnections
        });
    };

    return (
        <div className="relative h-screen">
            <Sidebar
                isSidebarOpen={isSidebarOpen}
                toggleSidebar={toggleSidebar}
                selectedTypes={selectedTypes}
                setSelectedTypes={setSelectedTypes}
                regexFilter={regexFilter}
                setRegexFilter={setRegexFilter}
                applyFilters={applyFilters}
            />

            {!isSidebarOpen && (
                <Button
                    variant="outline"
                    size="icon"
                    className="fixed top-4 left-4 z-40"
                    onClick={toggleSidebar}
                >
                    <Menu className="h-4 w-4" />
                </Button>
            )}

            <div className="fixed top-4 right-4 z-40 flex space-x-2">
                <Button
                    variant="outline"
                    size="icon"
                    onClick={() => handleZoom('in')}
                >
                    <Plus className="h-4 w-4" />
                </Button>
                <Button
                    variant="outline"
                    size="icon"
                    onClick={() => handleZoom('out')}
                >
                    <Minus className="h-4 w-4" />
                </Button>
            </div>

            <GraphVisualization
                svgRef={svgRef}
                zoomRef={zoomRef}
                graphData={graphData}
            />
        </div>
    );
};

export default NetworkGraph;
