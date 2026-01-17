'use client';

import { useEffect, useCallback } from 'react';
import {
  ReactFlow,
  Controls,
  Background,
  useNodesState,
  useEdgesState,
  addEdge,
  Connection,
  Edge,
  MarkerType,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { getApiUrl } from '@/services/api';

interface GraphNode {
  id: string;
  label: string;
  node_type: string;
}

interface GraphEdge {
  id: string;
  source: string;
  target: string;
  label?: string;
}

interface GraphResponse {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

export default function IotGraph() {
  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);

  const onConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge(params, eds)),
    [setEdges],
  );

  useEffect(() => {
    // Fetch graph data from backend
    fetch(`${getApiUrl()}/api/v1/iot/graph`)
      .then((res) => res.json())
      .then((data: GraphResponse) => {
        // Transform backend nodes to React Flow nodes
        const rfNodes = data.nodes.map((node, index) => ({
          id: node.id,
          position: { x: 100 + (index * 250), y: 100 + (index % 2 * 100) }, // Simple layout
          data: { label: node.label },
          style: {
            background: '#fff',
            border: '1px solid #777',
            borderRadius: '8px',
            padding: '10px',
            width: 150,
            textAlign: 'center',
            boxShadow: '0 4px 6px -1px rgb(0 0 0 / 0.1)'
          }
        }));

        // Transform backend edges to React Flow edges
        const rfEdges = data.edges.map((edge) => ({
          id: edge.id,
          source: edge.source,
          target: edge.target,
          label: edge.label,
          animated: true,
          markerEnd: {
            type: MarkerType.ArrowClosed,
          },
          style: { stroke: '#2563eb' }
        }));

        setNodes(rfNodes as any);
        setEdges(rfEdges as any);
      })
      .catch((err) => console.error("Failed to fetch IoT graph", err));
  }, [setNodes, setEdges]);

  return (
    <div className="h-full w-full">
      <div className="absolute top-4 left-4 z-10 bg-white/80 dark:bg-black/80 p-4 rounded shadow backdrop-blur">
        <h1 className="text-xl font-bold">IoT System Map</h1>
        <p className="text-sm text-gray-500">Visualizing device connectivity</p>
      </div>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        fitView
      >
        <Background />
        <Controls />
      </ReactFlow>
    </div>
  );
}
