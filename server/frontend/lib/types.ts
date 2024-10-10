export interface Node {
    id: string;
    os: string;
    type: string;
}

export interface Link {
    source: string | Node;
    target: string | Node;
}

export interface Connection {
    id: string;
    protocol: string;
    sourcePort: string;
    targetPort: string;
    description: string;
}

export interface GraphData {
    nodes: Node[];
    links: Link[];
    connections: Connection[];
}
