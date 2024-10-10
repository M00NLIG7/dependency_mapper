"use client"
import { useEffect, useRef, useState } from 'react';
import * as d3 from 'd3';
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Menu, X, Plus, Minus } from 'lucide-react';

const NetworkGraph = () => {
    const svgRef = useRef(null);
    const zoomRef = useRef(null);
    const [graphData, setGraphData] = useState({ nodes: [], links: [], connections: [] });
    const [isSidebarOpen, setIsSidebarOpen] = useState(false);
    const [selectedTypes, setSelectedTypes] = useState([]);
    const [regexFilter, setRegexFilter] = useState('');

    const icons = {
        Linux: '/static/assets/Linux.svg',
        Windows: '/static/assets/Windows.svg',
        Unknown: '/static/assets/Unknown.svg',
    };

    const getNodeIcon = (node) => icons[node.os] || icons.Unknown;

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

    useEffect(() => {
        if (!graphData.nodes.length) return;

        const svg = d3.select(svgRef.current);
        svg.selectAll('*').remove();
        const g = svg.append('g');
        let width = window.innerWidth;
        let height = window.innerHeight;

        zoomRef.current = d3
            .zoom()
            .scaleExtent([0.1, 4])
            .on('zoom', (event) => {
                g.attr('transform', event.transform);
            });

        svg.call(zoomRef.current);

        const linkGroup = g.append('g').attr('class', 'links');
        const nodeGroup = g.append('g').attr('class', 'nodes');
        const connectionGroup = g.append('g').attr('class', 'connections');

        const simulation = d3
            .forceSimulation()
            .force('link', d3.forceLink().id((d) => d.id).distance(150))
            .force('charge', d3.forceManyBody().strength(-800))
            .force('center', d3.forceCenter(width / 2, height / 2))
            .force('collision', d3.forceCollide().radius(60));

        const handleResize = () => {
            width = window.innerWidth;
            height = window.innerHeight;
            svg.attr('width', width).attr('height', height);
            simulation.force('center', d3.forceCenter(width / 2, height / 2));
            simulation.alpha(1).restart();
        };

        window.addEventListener('resize', handleResize);
        handleResize();

        const dragDrop = d3
            .drag()
            .on('start', (event, d) => {
                if (!event.active) simulation.alphaTarget(0.3).restart();
                d.fx = d.x;
                d.fy = d.y;
            })
            .on('drag', (event, d) => {
                d.fx = event.x;
                d.fy = event.y;
            })
            .on('end', (event, d) => {
                if (!event.active) simulation.alphaTarget(0);
                d.fx = null;
                d.fy = null;
            });

        const link = linkGroup
            .selectAll('.link')
            .data(graphData.links)
            .join('line')
            .attr('class', 'link')
            .attr('stroke', '#4A4A4A')
            .attr('stroke-width', 2)
            .attr('marker-end', 'url(#arrowhead)');

        const node = nodeGroup
            .selectAll('.node')
            .data(graphData.nodes)
            .join('g')
            .attr('class', 'node')
            .call(dragDrop);

        node
            .selectAll('image')
            .data((d) => [d])
            .join('image')
            .attr('xlink:href', getNodeIcon)
            .attr('width', 32)
            .attr('height', 32)
            .attr('x', -16)
            .attr('y', -16);

        node
            .selectAll('text')
            .data((d) => [d])
            .join('text')
            .text((d) => d.id)
            .attr('font-size', 10)
            .attr('dx', 20)
            .attr('dy', 4);

        const connection = connectionGroup
            .selectAll('.connection')
            .data(graphData.connections)
            .join('g')
            .attr('class', 'connection')
            .call(dragDrop);

        connection
            .selectAll('circle')
            .data((d) => [d])
            .join('circle')
            .attr('r', 15)
            .attr('class', 'connection-node');

        connection
            .selectAll('text')
            .data((d) => [d])
            .join('text')
            .text((d) => d.protocol)
            .attr('font-size', 8)
            .attr('text-anchor', 'middle')
            .attr('dy', -20);

        connection
            .selectAll('.port-text')
            .data((d) => [d])
            .join('text')
            .attr('class', 'port-text')
            .text((d) => `${d.sourcePort}->${d.targetPort}`)
            .attr('font-size', 8)
            .attr('text-anchor', 'middle')
            .attr('dy', 20);

        simulation.nodes(graphData.nodes.concat(graphData.connections)).on('tick', () => {
            link
                .attr('x1', (d) => d.source.x)
                .attr('y1', (d) => d.source.y)
                .attr('x2', (d) => d.target.x)
                .attr('y2', (d) => d.target.y);

            node.attr('transform', (d) => `translate(${d.x},${d.y})`);
            connection.attr('transform', (d) => `translate(${d.x},${d.y})`);
        });

        simulation.force('link').links(graphData.links);

        return () => {
            window.removeEventListener('resize', handleResize);
        };
    }, [graphData]);

    const toggleSidebar = () => {
        setIsSidebarOpen(!isSidebarOpen);
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
            <Card className={`fixed top-0 left-0 h-full w-64 shadow-lg transition-transform duration-300 ease-in-out z-50 ${isSidebarOpen ? 'translate-x-0' : '-translate-x-full'}`}>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                    <CardTitle className="text-sm font-medium">Filters</CardTitle>
                    <Button
                        variant="ghost"
                        size="icon"
                        onClick={toggleSidebar}
                        className="h-8 w-8 p-0"
                    >
                        <X className="h-4 w-4" />
                    </Button>
                </CardHeader>
                <CardContent>
                    <div className="space-y-4">
                        <div className="space-y-2">
                            <Label>Node Types</Label>
                            {['server', 'client', 'network'].map((type) => (
                                <div key={type} className="flex items-center space-x-2">
                                    <Checkbox
                                        id={type}
                                        checked={selectedTypes.includes(type)}
                                        onCheckedChange={(checked) => {
                                            setSelectedTypes(prev => 
                                                checked
                                                    ? [...prev, type]
                                                    : prev.filter(t => t !== type)
                                            );
                                        }}
                                    />
                                    <Label htmlFor={type}>{type.charAt(0).toUpperCase() + type.slice(1)}</Label>
                                </div>
                            ))}
                        </div>
                        <div className="space-y-2">
                            <Label htmlFor="regexFilter">Regex Filter</Label>
                            <Input
                                id="regexFilter"
                                placeholder="Enter regex..."
                                value={regexFilter}
                                onChange={(e) => setRegexFilter(e.target.value)}
                            />
                        </div>
                        <Button onClick={applyFilters} className="w-full">
                            Apply Filters
                        </Button>
                    </div>
                </CardContent>
            </Card>

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

            <svg ref={svgRef} className="w-full h-full"></svg>
        </div>
    );
};

export default NetworkGraph;
