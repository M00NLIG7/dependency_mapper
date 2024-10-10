import React, { useEffect, useMemo } from 'react';
import * as d3 from 'd3';

const GraphVisualization = ({ svgRef, zoomRef, graphData }) => {
    const icons = {
        Linux: '/static/assets/Linux.svg',
        Windows: '/static/assets/Windows.svg',
        Unknown: '/static/assets/Unknown.svg',
    };

    const getNodeIcon = (node) => icons[node.os] || icons.Unknown;

    // Calculate node sizes based on relationships
    const nodeSizes = useMemo(() => {
        const sizes = {};
        graphData.links.forEach(link => {
            sizes[link.source] = (sizes[link.source] || 0) + 1;
            sizes[link.target] = (sizes[link.target] || 0) + 1;
        });
        return Object.fromEntries(
            Object.entries(sizes).map(([id, count]) => [id, Math.max(32, Math.min(64, 32 + count * 4))])
        );
    }, [graphData.links]);

    useEffect(() => {
        if (!graphData.nodes.length) return;

        const svg = d3.select(svgRef.current);
        svg.selectAll('*').remove();
        const g = svg.append('g');
        let width = window.innerWidth;
        let height = window.innerHeight;

        svg.append("defs").append("marker")
            .attr("id", "arrowhead")
            .attr("viewBox", "0 -5 10 10")
            .attr("refX", 8)  // Adjusted to add padding
            .attr("refY", 0)
            .attr("markerWidth", 6)
            .attr("markerHeight", 6)
            .attr("orient", "auto")
            .append("path")
            .attr("d", "M0,-5L10,0L0,5")
            .attr("fill", "#4A4A4A");

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
            .force('link', d3.forceLink().id((d) => d.id).distance(200))
            .force('charge', d3.forceManyBody().strength(-1000))
            .force('center', d3.forceCenter(width / 2, height / 2))
            .force('collision', d3.forceCollide().radius(d => (nodeSizes[d.id] || 32) / 2 + 20));

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
            .join('path')
            .attr('class', 'link')
            .attr('stroke', '#4A4A4A')
            .attr('stroke-width', 2)
            .attr('fill', 'none')
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
            .attr('width', d => nodeSizes[d.id] || 32)
            .attr('height', d => nodeSizes[d.id] || 32)
            .attr('x', d => -(nodeSizes[d.id] || 32) / 2)
            .attr('y', d => -(nodeSizes[d.id] || 32) / 2);

        node
            .selectAll('text')
            .data((d) => [d])
            .join('text')
            .text((d) => d.id)
            .attr('font-size', 10)
            .attr('dx', d => (nodeSizes[d.id] || 32) / 2 + 5)
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
            link.attr('d', (d) => {
                const sourceX = d.source.x;
                const sourceY = d.source.y;
                const targetX = d.target.x;
                const targetY = d.target.y;

                const dx = targetX - sourceX;
                const dy = targetY - sourceY;
                const length = Math.sqrt(dx * dx + dy * dy);

                if (length === 0) return "M0,0L0,0";

                const unitDx = dx / length;
                const unitDy = dy / length;

                const sourceSize = (nodeSizes[d.source.id] || 32) / 2;
                const targetSize = (nodeSizes[d.target.id] || 32) / 2;

                const startX = sourceX + unitDx * (sourceSize + 2);
                const startY = sourceY + unitDy * (sourceSize + 2);
                const endX = targetX - unitDx * (targetSize + 6);
                const endY = targetY - unitDy * (targetSize + 6);

                return `M${startX},${startY}L${endX},${endY}`;
            });

            node.attr('transform', (d) => `translate(${d.x},${d.y})`);
            connection.attr('transform', (d) => `translate(${d.x},${d.y})`);
        });

        simulation.force('link').links(graphData.links);

        return () => {
            window.removeEventListener('resize', handleResize);
        };
    }, [graphData, nodeSizes]);

    return <svg ref={svgRef} className="w-full h-full"></svg>;
};

export default GraphVisualization;
