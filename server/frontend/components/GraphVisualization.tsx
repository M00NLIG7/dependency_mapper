import React, { useEffect } from 'react';
import * as d3 from 'd3';

const GraphVisualization = ({ svgRef, zoomRef, graphData }) => {
    const icons = {
        Linux: '/static/assets/Linux.svg',
        Windows: '/static/assets/Windows.svg',
        Unknown: '/static/assets/Unknown.svg',
    };

    const getNodeIcon = (node) => icons[node.os] || icons.Unknown;

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

    return <svg ref={svgRef} className="w-full h-full"></svg>;
};

export default GraphVisualization;

