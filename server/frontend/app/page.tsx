'use client';

import React, { useEffect, useRef, useState } from 'react';
import * as d3 from 'd3';
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Menu, X } from "lucide-react";

// Example data
const exampleData = {
    "nodes": [
        { "id": "Server1", "os": "Linux", "type": "server" },
        { "id": "Server2", "os": "Windows", "type": "server" },
        { "id": "Server3", "os": "Linux", "type": "server" },
        { "id": "Client1", "os": "Windows", "type": "client" },
        { "id": "Client2", "os": "Linux", "type": "client" },
        { "id": "Client3", "os": "Unknown", "type": "client" },
        { "id": "Router1", "os": "Unknown", "type": "network" },
        { "id": "Switch1", "os": "Unknown", "type": "network" }
    ],
    "connections": [
        { "id": "Conn1", "protocol": "HTTP", "sourcePort": 80, "targetPort": 8080, "description": "Web Traffic" },
        { "id": "Conn2", "protocol": "SSH", "sourcePort": 22, "targetPort": 22, "description": "Secure Shell" },
        { "id": "Conn3", "protocol": "FTP", "sourcePort": 21, "targetPort": 21, "description": "File Transfer" },
        { "id": "Conn4", "protocol": "SMTP", "sourcePort": 25, "targetPort": 25, "description": "Email" },
        { "id": "Conn5", "protocol": "DNS", "sourcePort": 53, "targetPort": 53, "description": "Domain Name System" }
    ],
    "edges": [
        { "source": "Client1", "target": "Server1", "connection": "Conn1" },
        { "source": "Client2", "target": "Server2", "connection": "Conn2" },
        { "source": "Client3", "target": "Server3", "connection": "Conn3" },
        { "source": "Server1", "target": "Server2", "connection": "Conn4" },
        { "source": "Server2", "target": "Server3", "connection": "Conn5" },
        { "source": "Client1", "target": "Router1", "connection": "Conn1" },
        { "source": "Router1", "target": "Switch1", "connection": "Conn1" },
        { "source": "Switch1", "target": "Server1", "connection": "Conn1" }
    ]
};

const NetworkGraph = () => {
    const graphRef = useRef(null);
    const [graphData, setGraphData] = useState({ nodes: [], connections: [], links: [] });
    const [isSidebarOpen, setIsSidebarOpen] = useState(false);
    const [selectedTypes, setSelectedTypes] = useState<string[]>([]);
    const [regexFilter, setRegexFilter] = useState('');

    useEffect(() => {
        // Use example data instead of fetching
        setGraphData({
            nodes: exampleData.nodes,
            connections: exampleData.connections,
            links: exampleData.edges.flatMap(edge => ([
                { source: edge.source, target: edge.connection },
                { source: edge.connection, target: edge.target }
            ]))
        });
    }, []);

    useEffect(() => {
        if (!graphData.nodes.length) return;

        const icons = {
            "Linux": "/images/icons/Linux.svg",
            "Windows": "/images/icons/Windows.svg",
            "Unknown": "/images/icons/Unknown.svg",
        };

        const getNodeIcon = (node) => icons[node.os] || icons["Unknown"];

        const width = window.innerWidth;
        const height = window.innerHeight;

        const svg = d3.select(graphRef.current)
            .append('svg')
            .attr('width', width)
            .attr('height', height);

        const g = svg.append("g");

        const zoom = d3.zoom()
            .scaleExtent([0.1, 4])
            .on("zoom", (event) => {
                g.attr("transform", event.transform);
            });

        svg.call(zoom);

        const linkGroup = g.append('g').attr('class', 'links');
        const nodeGroup = g.append('g').attr('class', 'nodes');
        const connectionGroup = g.append('g').attr('class', 'connections');

        const simulation = d3.forceSimulation()
            .force('link', d3.forceLink().id(d => d.id).distance(150))
            .force('charge', d3.forceManyBody().strength(-800))
            .force('center', d3.forceCenter(width / 2, height / 2))
            .force('collision', d3.forceCollide().radius(60));

        const dragDrop = d3.drag()
            .on('start', dragstarted)
            .on('drag', dragged)
            .on('end', dragended);

        function dragstarted(event, d) {
            if (!event.active) simulation.alphaTarget(0.3).restart();
            d.fx = d.x;
            d.fy = d.y;
        }

        function dragged(event, d) {
            d.fx = event.x;
            d.fy = event.y;
        }

        function dragended(event, d) {
            if (!event.active) simulation.alphaTarget(0);
            d.fx = null;
            d.fy = null;
        }

        const updateGraph = () => {
            const link = linkGroup.selectAll('.link')
                .data(graphData.links)
                .join('line')
                .attr('class', 'link')
                .attr('stroke', '#4A4A4A')
                .attr('stroke-width', 2);

            const node = nodeGroup.selectAll('.node')
                .data(graphData.nodes)
                .join('g')
                .attr('class', 'node')
                .call(dragDrop);

            node.selectAll('image')
                .data(d => [d])
                .join('image')
                .attr('xlink:href', getNodeIcon)
                .attr('width', 32)
                .attr('height', 32)
                .attr('x', -16)
                .attr('y', -16);

            node.selectAll('text')
                .data(d => [d])
                .join('text')
                .text(d => d.id)
                .attr('font-size', 10)
                .attr('dx', 20)
                .attr('dy', 4);

            const connection = connectionGroup.selectAll('.connection')
                .data(graphData.connections)
                .join('g')
                .attr('class', 'connection')
                .call(dragDrop);

            connection.selectAll('circle')
                .data(d => [d])
                .join('circle')
                .attr('r', 15)
                .attr('class', 'connection-node');

            connection.selectAll('text')
                .data(d => [d])
                .join('text')
                .text(d => d.protocol)
                .attr('font-size', 8)
                .attr('text-anchor', 'middle')
                .attr('dy', -20);

            connection.selectAll('.port-text')
                .data(d => [d])
                .join('text')
                .attr('class', 'port-text')
                .text(d => `${d.sourcePort}->${d.targetPort}`)
                .attr('font-size', 8)
                .attr('text-anchor', 'middle')
                .attr('dy', 20);

            simulation.nodes(graphData.nodes.concat(graphData.connections)).on('tick', () => {
                link
                    .attr('x1', d => d.source.x)
                    .attr('y1', d => d.source.y)
                    .attr('x2', d => {
                        const dx = d.target.x - d.source.x;
                        const dy = d.target.y - d.source.y;
                        const distance = Math.sqrt(dx * dx + dy * dy);
                        const targetRadius = d.target.protocol ? 15 : 16;
                        return d.target.x - (dx * (targetRadius + 10) / distance);
                    })
                    .attr('y2', d => {
                        const dx = d.target.x - d.source.x;
                        const dy = d.target.y - d.source.y;
                        const distance = Math.sqrt(dx * dx + dy * dy);
                        const targetRadius = d.target.protocol ? 15 : 16;
                        return d.target.y - (dy * (targetRadius + 10) / distance);
                    });

                node.attr('transform', d => `translate(${d.x},${d.y})`);
                connection.attr('transform', d => `translate(${d.x},${d.y})`);
            });

            simulation.force('link').links(graphData.links);
        };

        updateGraph();
        simulation.alpha(1).restart();

        return () => {
            svg.remove();
        };
    }, [graphData]);

    const toggleSidebar = () => {
        setIsSidebarOpen(!isSidebarOpen);
    };

    const handleZoom = (direction) => {
        const svg = d3.select(graphRef.current).select('svg');
        const zoom = d3.zoom().on("zoom", null);
        svg.transition().call(zoom.scaleBy, direction === 'in' ? 1.2 : 0.8);
    };

    const applyFilters = () => {
        let filteredNodes = exampleData.nodes;

        if (selectedTypes.length > 0) {
            filteredNodes = filteredNodes.filter(node => selectedTypes.includes(node.type));
        }

        if (regexFilter) {
            const regex = new RegExp(regexFilter);
            filteredNodes = filteredNodes.filter(node => regex.test(node.id));
        }

        const filteredLinks = exampleData.edges.filter(edge =>
            filteredNodes.some(node => node.id === edge.source) &&
            filteredNodes.some(node => node.id === edge.target)
        );

        setGraphData({
            nodes: filteredNodes,
            connections: exampleData.connections,
            links: filteredLinks.flatMap(edge => ([
                { source: edge.source, target: edge.connection },
                { source: edge.connection, target: edge.target }
            ]))
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
                    +
                </Button>
                <Button
                    variant="outline"
                    size="icon"
                    onClick={() => handleZoom('out')}
                >
                    -
                </Button>
            </div>

            <div id="graph" ref={graphRef} className="w-full h-full"></div>

            <style jsx>{`
                .link { stroke-opacity: 0.6; }
                .node text { font-size: 10px; }
                .node circle { stroke: #fff; stroke-width: 2px; }
                .connection-node { fill: #ffd700; stroke: #ff8c00; stroke-width: 2px; }
            `}</style>
        </div>
    );
};

export default function NetworkPage() {
    return (
        <div className="container mx-auto">
            <h1 className="text-2xl font-bold mb-4">Network Visualization</h1>
            <NetworkGraph />
        </div>
    );
}
