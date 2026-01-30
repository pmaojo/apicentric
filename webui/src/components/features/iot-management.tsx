'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { useToast } from '@/hooks/use-toast';
import { Loader2, Plus, Save, Trash2, Upload, RefreshCw } from 'lucide-react';
import { listTwins, getTwin, saveTwin, deleteTwin, uploadReplayData, getIotGraph, GraphResponse } from '@/services/api';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { ReactFlow, Controls, Background, useNodesState, useEdgesState, type Node, type Edge } from '@xyflow/react';
import '@xyflow/react/dist/style.css';

export function IoTManagement() {
  const [activeTab, setActiveTab] = React.useState('twins');
  const [twins, setTwins] = React.useState<string[]>([]);
  const [selectedTwin, setSelectedTwin] = React.useState<string | null>(null);
  const [yamlContent, setYamlContent] = React.useState('');
  const [loading, setLoading] = React.useState(false);
  const [uploading, setUploading] = React.useState(false);
  const { toast } = useToast();
  const fileInputRef = React.useRef<HTMLInputElement>(null);

  // React Flow state
  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);

  const fetchTwins = React.useCallback(async () => {
    try {
      setLoading(true);
      const data = await listTwins();
      setTwins(data);
    } catch (e) {
      toast({
        variant: 'destructive',
        title: 'Failed to fetch twins',
        description: e instanceof Error ? e.message : 'Unknown error',
      });
    } finally {
      setLoading(false);
    }
  }, [toast]);

  const fetchGraph = React.useCallback(async () => {
    try {
      setLoading(true);
      const data = await getIotGraph();
      // Map API response to React Flow format if needed (types match closely though)
      setNodes(data.nodes.map(n => ({ ...n, position: { x: n.position.x, y: n.position.y } })));
      setEdges(data.edges);
    } catch (e) {
      toast({
        variant: 'destructive',
        title: 'Failed to fetch graph',
        description: e instanceof Error ? e.message : 'Unknown error',
      });
    } finally {
      setLoading(false);
    }
  }, [setEdges, setNodes, toast]);

  React.useEffect(() => {
    fetchTwins();
  }, [fetchTwins]);

  React.useEffect(() => {
    if (activeTab === 'graph') {
      fetchGraph();
    }
  }, [activeTab, fetchGraph]);

  const handleSelectTwin = async (name: string) => {
    try {
      setLoading(true);
      const data = await getTwin(name);
      setSelectedTwin(name);
      setYamlContent(data.yaml);
    } catch (e) {
      toast({
        variant: 'destructive',
        title: 'Failed to fetch twin details',
        description: e instanceof Error ? e.message : 'Unknown error',
      });
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    if (!selectedTwin) return;
    try {
      setLoading(true);
      await saveTwin(selectedTwin, yamlContent);
      toast({ title: 'Twin saved' });
      fetchTwins();
    } catch (e) {
      toast({
        variant: 'destructive',
        title: 'Failed to save twin',
        description: e instanceof Error ? e.message : 'Unknown error',
      });
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async () => {
    if (!selectedTwin) return;
    if (!confirm(`Delete ${selectedTwin}?`)) return;
    try {
      setLoading(true);
      await deleteTwin(selectedTwin);
      toast({ title: 'Twin deleted' });
      setSelectedTwin(null);
      setYamlContent('');
      fetchTwins();
    } catch (e) {
      toast({
        variant: 'destructive',
        title: 'Failed to delete twin',
        description: e instanceof Error ? e.message : 'Unknown error',
      });
    } finally {
      setLoading(false);
    }
  };

  const handleCreate = () => {
    const name = prompt("Enter twin name:");
    if (!name) return;
    setSelectedTwin(name);
    setYamlContent(`twin:
  name: ${name}
  physics:
    - variable: temperature
      strategy: sine
      params:
        min: 20
        max: 30
  transports:
    - type: mqtt
      host: localhost
      port: 1883
      topic_prefix: iot/${name}
`);
  };

  const handleUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    if (!e.target.files || e.target.files.length === 0) return;
    const file = e.target.files[0];
    try {
      setUploading(true);
      const filename = await uploadReplayData(file);
      toast({ title: `Uploaded ${filename}` });
    } catch (e) {
      toast({
        variant: 'destructive',
        title: 'Upload failed',
        description: e instanceof Error ? e.message : 'Unknown error',
      });
    } finally {
      setUploading(false);
      if (fileInputRef.current) fileInputRef.current.value = '';
    }
  };

  return (
    <div className="h-[calc(100vh-120px)] flex flex-col">
      <Tabs value={activeTab} onValueChange={setActiveTab} className="h-full flex flex-col">
        <div className="flex justify-between items-center mb-4">
          <TabsList>
            <TabsTrigger value="twins">Twin Management</TabsTrigger>
            <TabsTrigger value="graph">System Map</TabsTrigger>
          </TabsList>
        </div>

        <TabsContent value="twins" className="flex-1 grid grid-cols-1 md:grid-cols-3 gap-4 overflow-hidden">
          <Card className="md:col-span-1 flex flex-col">
            <CardHeader>
              <div className="flex justify-between items-center">
                <CardTitle>Twins</CardTitle>
                <Button size="sm" onClick={handleCreate}><Plus className="h-4 w-4" /></Button>
              </div>
            </CardHeader>
            <CardContent className="flex-1 overflow-y-auto">
              {loading && !selectedTwin ? (
                <div className="flex justify-center p-4"><Loader2 className="animate-spin" /></div>
              ) : (
                <div className="space-y-2">
                  {twins.map(t => (
                    <div
                      key={t}
                      role="button"
                      tabIndex={0}
                      className={`p-2 rounded cursor-pointer hover:bg-accent ${selectedTwin === t ? 'bg-accent' : ''} focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2`}
                      onClick={() => handleSelectTwin(t)}
                      onKeyDown={(e) => {
                        if (e.key === 'Enter' || e.key === ' ') {
                          e.preventDefault();
                          handleSelectTwin(t);
                        }
                      }}
                      aria-current={selectedTwin === t ? 'true' : undefined}
                      aria-label={`Select twin ${t}`}
                    >
                      {t}
                    </div>
                  ))}
                  {twins.length === 0 && <div className="text-muted-foreground p-2">No twins found</div>}
                </div>
              )}
            </CardContent>
          </Card>

          <Card className="md:col-span-2 flex flex-col h-full">
            <CardHeader>
              <div className="flex justify-between items-center">
                 <CardTitle>{selectedTwin || 'Select a Twin'}</CardTitle>
                 {selectedTwin && (
                   <div className="flex gap-2">
                     <input
                       type="file"
                       accept=".csv"
                       ref={fileInputRef}
                       className="hidden"
                       onChange={handleUpload}
                     />
                     <Button variant="outline" size="sm" onClick={() => fileInputRef.current?.click()} disabled={uploading}>
                       {uploading ? <Loader2 className="h-4 w-4 animate-spin" /> : <Upload className="h-4 w-4 mr-2" />}
                       Upload CSV
                     </Button>
                     <Button variant="outline" size="sm" onClick={handleSave} disabled={loading}>
                       <Save className="h-4 w-4 mr-2" /> Save
                     </Button>
                     <Button variant="destructive" size="sm" onClick={handleDelete} disabled={loading}>
                       <Trash2 className="h-4 w-4 mr-2" /> Delete
                     </Button>
                   </div>
                 )}
              </div>
            </CardHeader>
            <CardContent className="flex-1 p-0 relative overflow-hidden">
              {selectedTwin ? (
                <Textarea
                  className="absolute inset-0 w-full h-full font-mono p-4 border-0 focus-visible:ring-0 resize-none rounded-b-lg"
                  value={yamlContent}
                  onChange={(e) => setYamlContent(e.target.value)}
                />
              ) : (
                 <div className="flex items-center justify-center h-full text-muted-foreground">
                   Select or create a twin to edit
                 </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="graph" className="flex-1 h-full border rounded-lg bg-background relative">
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            fitView
          >
            <Background />
            <Controls />
          </ReactFlow>
          <div className="absolute top-4 right-4 z-10">
            <Button size="sm" variant="outline" onClick={fetchGraph} disabled={loading}>
                <RefreshCw className={`h-4 w-4 mr-2 ${loading ? 'animate-spin' : ''}`} />
                Refresh
            </Button>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}
