import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Server, Plug, PlayCircle, Copy, Check, AlertCircle,
  Loader2, Plus, X, ChevronDown, ChevronRight
} from 'lucide-react';
import { toast, invokeWithToast } from '../lib/toast';

interface McpServer {
  id: string;
  name: string;
  transport: 'stdio' | 'http';
  command?: string;
  args?: string[];
  url?: string;
  status: 'connected' | 'disconnected' | 'connecting' | 'error';
}

interface McpTool {
  name: string;
  description: string;
  parameters: Record<string, {
    type: string;
    description?: string;
    required?: boolean;
  }>;
}

interface McpConnection {
  server_id: string;
  server_name: string;
  tools: McpTool[];
}

export function MCPToolsBrowser() {
  const [servers, setServers] = useState<McpServer[]>([]);
  const [connections, setConnections] = useState<McpConnection[]>([]);
  const [selectedServer, setSelectedServer] = useState<string | null>(null);
  const [selectedTool, setSelectedTool] = useState<McpTool | null>(null);
  const [toolInput, setToolInput] = useState<Record<string, string>>({});
  const [toolResult, setToolResult] = useState<string | null>(null);
  const [isExecuting, setIsExecuting] = useState(false);
  const [showAddServer, setShowAddServer] = useState(false);
  const [newServer, setNewServer] = useState({
    name: '',
    transport: 'stdio' as 'stdio' | 'http',
    command: '',
    args: '',
    url: '',
  });

  useEffect(() => {
    loadConnections();
  }, []);

  const loadConnections = async () => {
    try {
      const conns = await invoke<McpConnection[]>('mcp_list_connections');
      setConnections(conns);
    } catch (error) {
      console.error('Failed to load MCP connections:', error);
    }
  };

  const handleConnect = async (server: McpServer) => {
    try {
      await invokeWithToast(
        'mcp_connect',
        {
          serverId: server.id,
          transport: server.transport,
          command: server.command,
          args: server.args,
          url: server.url,
        },
        {
          loading: `Connecting to ${server.name}...`,
          success: `Connected to ${server.name}`,
          error: 'Failed to connect to MCP server',
        }
      );
      await loadConnections();
      setSelectedServer(server.id);
    } catch (error) {
      console.error('Connection failed:', error);
    }
  };

  const handleDisconnect = async (serverId: string) => {
    try {
      await invokeWithToast('mcp_disconnect', { serverId }, {
        loading: 'Disconnecting...',
        success: 'Disconnected',
        error: 'Failed to disconnect',
      });
      await loadConnections();
      if (selectedServer === serverId) {
        setSelectedServer(null);
      }
    } catch (error) {
      console.error('Disconnect failed:', error);
    }
  };

  const handleExecuteTool = async () => {
    if (!selectedTool || !selectedServer) return;

    setIsExecuting(true);
    setToolResult(null);

    try {
      const result = await invoke<string>('mcp_call_tool', {
        serverId: selectedServer,
        toolName: selectedTool.name,
        parameters: toolInput,
      });

      setToolResult(result);
      toast.success('Tool executed successfully');
    } catch (error) {
      toast.error('Tool execution failed', String(error));
      setToolResult(`Error: ${error}`);
    } finally {
      setIsExecuting(false);
    }
  };

  const handleAddServer = () => {
    const server: McpServer = {
      id: `server-${Date.now()}`,
      name: newServer.name,
      transport: newServer.transport,
      command: newServer.transport === 'stdio' ? newServer.command : undefined,
      args: newServer.transport === 'stdio' && newServer.args
        ? newServer.args.split(' ')
        : undefined,
      url: newServer.transport === 'http' ? newServer.url : undefined,
      status: 'disconnected',
    };

    setServers([...servers, server]);
    setShowAddServer(false);
    setNewServer({
      name: '',
      transport: 'stdio',
      command: '',
      args: '',
      url: '',
    });
  };

  const selectedConnection = connections.find(c => c.server_id === selectedServer);
  const availableTools = selectedConnection?.tools || [];

  return (
    <div className="flex h-full">
      {/* Sidebar - Servers */}
      <div className="w-80 border-r border-zinc-700 bg-zinc-900/50 flex flex-col">
        <div className="p-4 border-b border-zinc-700">
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-lg font-semibold text-white">MCP Servers</h3>
            <button
              onClick={() => setShowAddServer(true)}
              className="p-2 rounded-lg bg-sky-400/60 hover:bg-sky-400/80 transition-colors"
            >
              <Plus className="w-4 h-4 text-white" />
            </button>
          </div>
          <p className="text-sm text-zinc-400">
            Connect to Model Context Protocol servers
          </p>
        </div>

        <div className="flex-1 overflow-y-auto p-4 space-y-2">
          {servers.length === 0 && connections.length === 0 ? (
            <div className="text-center py-8 text-zinc-500">
              <Server className="w-12 h-12 mx-auto mb-2 opacity-50" />
              <p className="text-sm">No servers configured</p>
              <p className="text-xs mt-1">Click + to add a server</p>
            </div>
          ) : (
            <>
              {connections.map(conn => (
                <div
                  key={conn.server_id}
                  onClick={() => setSelectedServer(conn.server_id)}
                  className={`p-3 rounded-lg border cursor-pointer transition-colors ${
                    selectedServer === conn.server_id
                      ? 'border-sky-400/60 bg-sky-400/10'
                      : 'border-zinc-700 bg-zinc-800/50 hover:border-zinc-600'
                  }`}
                >
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center space-x-2">
                      <div className="w-2 h-2 rounded-full bg-green-400" />
                      <span className="font-medium text-white">
                        {conn.server_name}
                      </span>
                    </div>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDisconnect(conn.server_id);
                      }}
                      className="p-1 hover:bg-zinc-700 rounded"
                    >
                      <X className="w-4 h-4 text-zinc-400" />
                    </button>
                  </div>
                  <p className="text-xs text-zinc-400">
                    {conn.tools.length} tools available
                  </p>
                </div>
              ))}

              {servers.map(server => (
                <div
                  key={server.id}
                  className="p-3 rounded-lg border border-zinc-700 bg-zinc-800/50"
                >
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center space-x-2">
                      <div className="w-2 h-2 rounded-full bg-zinc-500" />
                      <span className="font-medium text-white">{server.name}</span>
                    </div>
                  </div>
                  <div className="flex items-center space-x-2">
                    <button
                      onClick={() => handleConnect(server)}
                      className="flex-1 px-3 py-1 text-xs bg-sky-400/60 hover:bg-sky-400/80 text-white rounded transition-colors"
                    >
                      Connect
                    </button>
                    <button
                      onClick={() => setServers(servers.filter(s => s.id !== server.id))}
                      className="p-1 hover:bg-zinc-700 rounded"
                    >
                      <X className="w-4 h-4 text-zinc-400" />
                    </button>
                  </div>
                </div>
              ))}
            </>
          )}
        </div>
      </div>

      {/* Main Content - Tools */}
      <div className="flex-1 flex">
        {/* Tools List */}
        <div className="w-80 border-r border-zinc-700 bg-zinc-900/30 flex flex-col">
          <div className="p-4 border-b border-zinc-700">
            <h3 className="text-lg font-semibold text-white">Available Tools</h3>
            {selectedConnection && (
              <p className="text-sm text-zinc-400 mt-1">
                {availableTools.length} tools from {selectedConnection.server_name}
              </p>
            )}
          </div>

          <div className="flex-1 overflow-y-auto p-4 space-y-2">
            {!selectedServer ? (
              <div className="text-center py-8 text-zinc-500">
                <Plug className="w-12 h-12 mx-auto mb-2 opacity-50" />
                <p className="text-sm">Select a server to view tools</p>
              </div>
            ) : availableTools.length === 0 ? (
              <div className="text-center py-8 text-zinc-500">
                <AlertCircle className="w-12 h-12 mx-auto mb-2 opacity-50" />
                <p className="text-sm">No tools available</p>
              </div>
            ) : (
              availableTools.map(tool => (
                <div
                  key={tool.name}
                  onClick={() => {
                    setSelectedTool(tool);
                    setToolInput({});
                    setToolResult(null);
                  }}
                  className={`p-3 rounded-lg border cursor-pointer transition-colors ${
                    selectedTool?.name === tool.name
                      ? 'border-sky-400/60 bg-sky-400/10'
                      : 'border-zinc-700 bg-zinc-800/50 hover:border-zinc-600'
                  }`}
                >
                  <div className="font-medium text-white mb-1">{tool.name}</div>
                  <p className="text-xs text-zinc-400 line-clamp-2">
                    {tool.description}
                  </p>
                </div>
              ))
            )}
          </div>
        </div>

        {/* Tool Executor */}
        <div className="flex-1 flex flex-col">
          {!selectedTool ? (
            <div className="flex items-center justify-center h-full text-zinc-500">
              <div className="text-center">
                <PlayCircle className="w-16 h-16 mx-auto mb-4 opacity-50" />
                <p className="text-lg">Select a tool to execute</p>
              </div>
            </div>
          ) : (
            <>
              {/* Tool Details */}
              <div className="p-6 border-b border-zinc-700 bg-zinc-900/30">
                <h2 className="text-2xl font-bold text-white mb-2">
                  {selectedTool.name}
                </h2>
                <p className="text-zinc-400 mb-4">{selectedTool.description}</p>

                {/* Parameters */}
                <div className="space-y-3">
                  {Object.entries(selectedTool.parameters).map(([name, param]) => (
                    <div key={name}>
                      <label className="block text-sm font-medium text-zinc-300 mb-2">
                        {name}
                        {param.required && (
                          <span className="text-red-400 ml-1">*</span>
                        )}
                      </label>
                      {param.description && (
                        <p className="text-xs text-zinc-500 mb-2">
                          {param.description}
                        </p>
                      )}
                      <input
                        type="text"
                        value={toolInput[name] || ''}
                        onChange={(e) =>
                          setToolInput({ ...toolInput, [name]: e.target.value })
                        }
                        placeholder={`Enter ${name}`}
                        className="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white placeholder-zinc-500 focus:outline-none focus:ring-2 focus:ring-sky-400/60"
                      />
                    </div>
                  ))}
                </div>

                {/* Execute Button */}
                <button
                  onClick={handleExecuteTool}
                  disabled={isExecuting}
                  className="mt-4 w-full flex items-center justify-center space-x-2 px-4 py-3 bg-sky-400/60 hover:bg-sky-400/80 disabled:opacity-50 text-white rounded-lg transition-colors"
                >
                  {isExecuting ? (
                    <>
                      <Loader2 className="w-4 h-4 animate-spin" />
                      <span>Executing...</span>
                    </>
                  ) : (
                    <>
                      <PlayCircle className="w-4 h-4" />
                      <span>Execute Tool</span>
                    </>
                  )}
                </button>
              </div>

              {/* Results */}
              <div className="flex-1 overflow-y-auto p-6">
                <h3 className="text-lg font-semibold text-white mb-3">Result</h3>
                {!toolResult ? (
                  <div className="text-zinc-500 text-sm">
                    Results will appear here after execution
                  </div>
                ) : (
                  <div className="bg-zinc-900 border border-zinc-800 rounded-lg p-4">
                    <pre className="text-sm text-zinc-300 whitespace-pre-wrap font-mono">
                      {toolResult}
                    </pre>
                  </div>
                )}
              </div>
            </>
          )}
        </div>
      </div>

      {/* Add Server Modal */}
      {showAddServer && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-6">
          <div className="bg-zinc-900 border border-zinc-700 rounded-lg max-w-2xl w-full">
            <div className="p-6 border-b border-zinc-700">
              <h2 className="text-2xl font-bold text-white">Add MCP Server</h2>
            </div>

            <div className="p-6 space-y-4">
              <div>
                <label className="block text-sm font-medium text-zinc-300 mb-2">
                  Server Name
                </label>
                <input
                  type="text"
                  value={newServer.name}
                  onChange={(e) => setNewServer({ ...newServer, name: e.target.value })}
                  placeholder="My MCP Server"
                  className="w-full px-4 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-sky-400/60"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-zinc-300 mb-2">
                  Transport Type
                </label>
                <select
                  value={newServer.transport}
                  onChange={(e) =>
                    setNewServer({
                      ...newServer,
                      transport: e.target.value as 'stdio' | 'http',
                    })
                  }
                  className="w-full px-4 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-sky-400/60"
                >
                  <option value="stdio">stdio (Local Command)</option>
                  <option value="http">HTTP (Remote Server)</option>
                </select>
              </div>

              {newServer.transport === 'stdio' ? (
                <>
                  <div>
                    <label className="block text-sm font-medium text-zinc-300 mb-2">
                      Command
                    </label>
                    <input
                      type="text"
                      value={newServer.command}
                      onChange={(e) =>
                        setNewServer({ ...newServer, command: e.target.value })
                      }
                      placeholder="npx @modelcontextprotocol/server-example"
                      className="w-full px-4 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-sky-400/60"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-zinc-300 mb-2">
                      Arguments (space-separated)
                    </label>
                    <input
                      type="text"
                      value={newServer.args}
                      onChange={(e) =>
                        setNewServer({ ...newServer, args: e.target.value })
                      }
                      placeholder="--port 3000"
                      className="w-full px-4 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-sky-400/60"
                    />
                  </div>
                </>
              ) : (
                <div>
                  <label className="block text-sm font-medium text-zinc-300 mb-2">
                    URL
                  </label>
                  <input
                    type="text"
                    value={newServer.url}
                    onChange={(e) =>
                      setNewServer({ ...newServer, url: e.target.value })
                    }
                    placeholder="http://localhost:3000/mcp"
                    className="w-full px-4 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-sky-400/60"
                  />
                </div>
              )}
            </div>

            <div className="p-6 border-t border-zinc-700 flex justify-end space-x-3">
              <button
                onClick={() => setShowAddServer(false)}
                className="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-white rounded-lg transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleAddServer}
                disabled={!newServer.name}
                className="px-4 py-2 bg-sky-400/60 hover:bg-sky-400/80 disabled:opacity-50 text-white rounded-lg transition-colors"
              >
                Add Server
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
