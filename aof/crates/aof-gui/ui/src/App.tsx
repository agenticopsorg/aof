import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import {
  Terminal, Cpu, Network, Settings as SettingsIcon, Play, Square,
  FileCode, Check, AlertCircle, Loader2, Trash2,
  RefreshCw, Brain, MessageCircle, BarChart3
} from 'lucide-react';
import { Toaster } from 'sonner';
import { Settings } from './components/Settings';
import { AgentTemplates } from './components/AgentTemplates';
import { MCPToolsBrowser } from './components/MCPToolsBrowser';
import { MemoryViewer } from './components/MemoryViewer';
import { PlatformIntegrations } from './components/PlatformIntegrations';
import { SystemMonitoring } from './components/SystemMonitoring';
import { MarkdownRenderer } from './components/MarkdownRenderer';
import './lib/toast';
import './App.css';

// Types
interface AppInfo {
  version: string;
  name: string;
  aof_core_version: string;
}

interface AgentStatus {
  agent_id: string;
  name: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'stopped';
  output: string[];      // Status logs (legacy)
  response?: string;     // The actual LLM response
  metadata?: {
    input_tokens: number;
    output_tokens: number;
    execution_time_ms: number;
    tool_calls: number;
    model?: string;
  };
  started_at?: string;
  finished_at?: string;
  error?: string;
}

interface ValidationResult {
  valid: boolean;
  errors: Array<{ field: string; message: string; line?: number }>;
  warnings: string[];
  config?: {
    name: string;
    model: string;
    tools_count: number;
    max_iterations: number;
    temperature: number;
    has_system_prompt: boolean;
    has_memory: boolean;
  };
}

// Tab type
type TabType = 'agents' | 'config' | 'templates' | 'mcp' | 'memory' | 'integrations' | 'monitoring' | 'settings';

function App() {
  // App state
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [activeTab, setActiveTab] = useState<TabType>('agents');

  // Agent state
  const [agents, setAgents] = useState<AgentStatus[]>([]);
  const [selectedAgent, setSelectedAgent] = useState<string | null>(null);

  // Config state
  const [configYaml, setConfigYaml] = useState<string>('');
  const [validation, setValidation] = useState<ValidationResult | null>(null);
  const [agentInput, setAgentInput] = useState<string>('');
  const [isRunning, setIsRunning] = useState(false);

  // Load app info on mount
  useEffect(() => {
    invoke<AppInfo>('get_version').then(setAppInfo).catch(console.error);
    loadAgents();
    loadExampleConfig();
  }, []);

  // Set up event listeners
  useEffect(() => {
    const listeners: Promise<UnlistenFn>[] = [];

    listeners.push(
      listen<{ agent_id: string; content: string }>('agent-output', (event) => {
        setAgents(prev => prev.map(a =>
          a.agent_id === event.payload.agent_id
            ? { ...a, output: [...a.output, event.payload.content] }
            : a
        ));
      })
    );

    listeners.push(
      listen<{ agent_id: string; result: string; execution_time_ms: number; metadata?: AgentStatus['metadata'] }>('agent-completed', (event) => {
        // Update the agent directly with the response from the event
        setAgents(prev => prev.map(a =>
          a.agent_id === event.payload.agent_id
            ? {
                ...a,
                status: 'completed' as const,
                response: event.payload.result,
                metadata: event.payload.metadata || a.metadata,
              }
            : a
        ));
        setIsRunning(false);
        // Also refresh the full list to get any other updates
        loadAgents();
      })
    );

    listeners.push(
      listen<{ agent_id: string }>('agent-stopped', () => {
        loadAgents();
        setIsRunning(false);
      })
    );

    listeners.push(
      listen<{ agent_id: string; error: string }>('agent-error', (event) => {
        console.error('Agent error:', event.payload.error);
        loadAgents();
        setIsRunning(false);
      })
    );

    return () => {
      listeners.forEach(p => p.then(unlisten => unlisten()));
    };
  }, []);

  const loadAgents = async () => {
    try {
      const list = await invoke<AgentStatus[]>('agent_list');
      setAgents(list);
    } catch (error) {
      console.error('Failed to load agents:', error);
    }
  };

  const loadExampleConfig = async () => {
    try {
      const example = await invoke<string>('config_generate_example');
      setConfigYaml(example);
      validateConfig(example);
    } catch (error) {
      console.error('Failed to load example config:', error);
    }
  };

  const handleLoadTemplate = (yaml: string) => {
    setConfigYaml(yaml);
    validateConfig(yaml);
    setActiveTab('config');
  };

  const validateConfig = useCallback(async (yaml: string) => {
    try {
      const result = await invoke<ValidationResult>('config_validate', { yamlContent: yaml });
      setValidation(result);
    } catch (error) {
      console.error('Validation error:', error);
    }
  }, []);

  const handleConfigChange = (value: string) => {
    setConfigYaml(value);
    validateConfig(value);
  };

  const handleRunAgent = async () => {
    if (!validation?.valid || !agentInput.trim()) return;

    setIsRunning(true);
    try {
      const response = await invoke<{ agent_id: string; name: string }>('agent_run', {
        request: {
          config_yaml: configYaml,
          input: agentInput,
        }
      });
      setSelectedAgent(response.agent_id);
      setActiveTab('agents');
      loadAgents();
    } catch (error) {
      console.error('Failed to run agent:', error);
      setIsRunning(false);
    }
  };

  const handleStopAgent = async (agentId: string) => {
    try {
      await invoke('agent_stop', { agentId });
      loadAgents();
    } catch (error) {
      console.error('Failed to stop agent:', error);
    }
  };

  const handleClearCompleted = async () => {
    try {
      await invoke('agent_clear_completed');
      loadAgents();
      setSelectedAgent(null);
    } catch (error) {
      console.error('Failed to clear completed:', error);
    }
  };

  const selectedAgentData = agents.find(a => a.agent_id === selectedAgent);

  return (
    <div className="min-h-screen bg-gradient-to-br from-zinc-900 via-zinc-800 to-zinc-900">
      {/* Toast Container */}
      <Toaster
        position="top-right"
        toastOptions={{
          style: {
            background: '#27272a',
            color: '#fff',
            border: '1px solid #3f3f46',
          },
        }}
      />
      {/* Header */}
      <header className="border-b border-slate-700 bg-slate-900/50 backdrop-blur sticky top-0 z-10">
        <div className="container mx-auto px-6 py-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-3">
              <Terminal className="w-7 h-7 text-blue-400" />
              <div>
                <h1 className="text-xl font-bold text-white">AOF Desktop</h1>
                {appInfo && (
                  <p className="text-xs text-slate-400">
                    v{appInfo.version} • Core v{appInfo.aof_core_version}
                  </p>
                )}
              </div>
            </div>
            <div className="flex items-center space-x-2">
              <button
                onClick={loadAgents}
                className="p-2 rounded-lg hover:bg-zinc-800 transition-colors"
                title="Refresh"
              >
                <RefreshCw className="w-4 h-4 text-zinc-400" />
              </button>
            </div>
          </div>
        </div>
      </header>

      {/* Tab Navigation */}
      <div className="border-b border-slate-700 bg-slate-900/30">
        <div className="container mx-auto px-6">
          <nav className="flex space-x-1">
            {[
              { id: 'agents' as TabType, label: 'Agents', icon: Cpu },
              { id: 'config' as TabType, label: 'Configuration', icon: FileCode },
              { id: 'templates' as TabType, label: 'Templates', icon: FileCode },
              { id: 'mcp' as TabType, label: 'MCP Tools', icon: Network },
              { id: 'memory' as TabType, label: 'Memory', icon: Brain },
              { id: 'integrations' as TabType, label: 'Integrations', icon: MessageCircle },
              { id: 'monitoring' as TabType, label: 'Monitoring', icon: BarChart3 },
              { id: 'settings' as TabType, label: 'Settings', icon: SettingsIcon },
            ].map(({ id, label, icon: Icon }) => (
              <button
                key={id}
                onClick={() => setActiveTab(id)}
                className={`flex items-center space-x-2 px-4 py-3 text-sm font-medium transition-colors border-b-2 -mb-px ${
                  activeTab === id
                    ? 'text-blue-400 border-blue-400'
                    : 'text-slate-400 border-transparent hover:text-slate-200'
                }`}
              >
                <Icon className="w-4 h-4" />
                <span>{label}</span>
              </button>
            ))}
          </nav>
        </div>
      </div>

      {/* Main Content */}
      <main className="container mx-auto px-6 py-6">
        {/* Agents Tab */}
        {activeTab === 'agents' && (
          <div className="grid grid-cols-3 gap-6">
            {/* Agent List */}
            <div className="col-span-1 bg-slate-800/50 backdrop-blur rounded-lg border border-slate-700">
              <div className="p-4 border-b border-slate-700 flex items-center justify-between">
                <h2 className="text-lg font-semibold text-white">Active Agents</h2>
                <button
                  onClick={handleClearCompleted}
                  className="text-xs text-slate-400 hover:text-slate-200 flex items-center space-x-1"
                >
                  <Trash2 className="w-3 h-3" />
                  <span>Clear</span>
                </button>
              </div>
              <div className="p-2 max-h-96 overflow-y-auto">
                {agents.length === 0 ? (
                  <p className="text-slate-500 text-center py-8">No agents running</p>
                ) : (
                  agents.map(agent => (
                    <button
                      key={agent.agent_id}
                      onClick={() => setSelectedAgent(agent.agent_id)}
                      className={`w-full p-3 rounded-lg text-left transition-colors mb-1 ${
                        selectedAgent === agent.agent_id
                          ? 'bg-blue-600/20 border border-blue-500'
                          : 'hover:bg-slate-700/50 border border-transparent'
                      }`}
                    >
                      <div className="flex items-center justify-between">
                        <span className="text-white font-medium">{agent.name}</span>
                        <StatusBadge status={agent.status} />
                      </div>
                      {agent.metadata && (
                        <p className="text-xs text-slate-400 mt-1">
                          {agent.metadata.execution_time_ms}ms • {agent.metadata.model}
                        </p>
                      )}
                    </button>
                  ))
                )}
              </div>
            </div>

            {/* Agent Output */}
            <div className="col-span-2 bg-slate-800/50 backdrop-blur rounded-lg border border-slate-700">
              <div className="p-4 border-b border-slate-700 flex items-center justify-between">
                <h2 className="text-lg font-semibold text-white">
                  {selectedAgentData ? selectedAgentData.name : 'Agent Output'}
                </h2>
                {selectedAgentData?.status === 'running' && (
                  <button
                    onClick={() => handleStopAgent(selectedAgentData.agent_id)}
                    className="flex items-center space-x-1 px-3 py-1 bg-red-600 hover:bg-red-700 text-white text-sm rounded-lg transition-colors"
                  >
                    <Square className="w-3 h-3" />
                    <span>Stop</span>
                  </button>
                )}
              </div>
              <div className="p-4 h-[500px] overflow-y-auto">
                {selectedAgentData ? (
                  <div>
                    {/* Show actual LLM response with markdown rendering */}
                    {selectedAgentData.response ? (
                      <MarkdownRenderer content={selectedAgentData.response} />
                    ) : selectedAgentData.status === 'running' ? (
                      <div className="flex items-center space-x-2 text-blue-400">
                        <Loader2 className="w-4 h-4 animate-spin" />
                        <span>Processing request...</span>
                      </div>
                    ) : selectedAgentData.error ? (
                      <div className="p-4 bg-red-900/20 border border-red-700 rounded-lg">
                        <p className="text-red-400 font-medium">Error</p>
                        <p className="text-red-300 mt-1">{selectedAgentData.error}</p>
                      </div>
                    ) : (
                      <p className="text-slate-500">Waiting for response...</p>
                    )}
                  </div>
                ) : (
                  <p className="text-slate-500">Select an agent to view output</p>
                )}
              </div>
            </div>
          </div>
        )}

        {/* Configuration Tab */}
        {activeTab === 'config' && (
          <div className="grid grid-cols-2 gap-6">
            {/* YAML Editor */}
            <div className="bg-slate-800/50 backdrop-blur rounded-lg border border-slate-700">
              <div className="p-4 border-b border-slate-700 flex items-center justify-between">
                <h2 className="text-lg font-semibold text-white">Agent Configuration</h2>
                <button
                  onClick={loadExampleConfig}
                  className="text-xs text-slate-400 hover:text-slate-200"
                >
                  Reset to Example
                </button>
              </div>
              <div className="p-4">
                <textarea
                  value={configYaml}
                  onChange={(e) => handleConfigChange(e.target.value)}
                  className="w-full h-64 bg-slate-900 border border-slate-600 rounded-lg p-4 font-mono text-sm text-slate-300 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none"
                  placeholder="# Enter agent YAML configuration..."
                  spellCheck={false}
                />

                {/* Validation Status */}
                <div className="mt-4">
                  {validation && (
                    <div className={`p-3 rounded-lg ${
                      validation.valid
                        ? 'bg-green-900/20 border border-green-700'
                        : 'bg-red-900/20 border border-red-700'
                    }`}>
                      <div className="flex items-center space-x-2">
                        {validation.valid ? (
                          <>
                            <Check className="w-4 h-4 text-green-400" />
                            <span className="text-green-400 font-medium">Valid Configuration</span>
                          </>
                        ) : (
                          <>
                            <AlertCircle className="w-4 h-4 text-red-400" />
                            <span className="text-red-400 font-medium">Invalid Configuration</span>
                          </>
                        )}
                      </div>
                      {validation.errors.length > 0 && (
                        <ul className="mt-2 text-sm text-red-300">
                          {validation.errors.map((err, i) => (
                            <li key={i}>• {err.field}: {err.message}</li>
                          ))}
                        </ul>
                      )}
                      {validation.warnings.length > 0 && (
                        <ul className="mt-2 text-sm text-yellow-300">
                          {validation.warnings.map((warn, i) => (
                            <li key={i}>⚠ {warn}</li>
                          ))}
                        </ul>
                      )}
                    </div>
                  )}
                </div>
              </div>
            </div>

            {/* Run Panel */}
            <div className="bg-slate-800/50 backdrop-blur rounded-lg border border-slate-700">
              <div className="p-4 border-b border-slate-700">
                <h2 className="text-lg font-semibold text-white">Run Agent</h2>
              </div>
              <div className="p-4">
                {/* Config Summary */}
                {validation?.config && (
                  <div className="mb-4 p-4 bg-slate-900/50 rounded-lg border border-slate-700">
                    <h3 className="text-sm font-semibold text-white mb-2">Configuration Summary</h3>
                    <div className="grid grid-cols-2 gap-2 text-sm">
                      <div>
                        <span className="text-slate-400">Name:</span>
                        <span className="text-white ml-2">{validation.config.name}</span>
                      </div>
                      <div>
                        <span className="text-slate-400">Model:</span>
                        <span className="text-white ml-2">{validation.config.model}</span>
                      </div>
                      <div>
                        <span className="text-slate-400">Tools:</span>
                        <span className="text-white ml-2">{validation.config.tools_count}</span>
                      </div>
                      <div>
                        <span className="text-slate-400">Temperature:</span>
                        <span className="text-white ml-2">{validation.config.temperature}</span>
                      </div>
                    </div>
                  </div>
                )}

                {/* Input */}
                <div className="mb-4">
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Agent Input / Query
                  </label>
                  <textarea
                    value={agentInput}
                    onChange={(e) => setAgentInput(e.target.value)}
                    className="w-full h-32 bg-slate-900 border border-slate-600 rounded-lg p-4 text-sm text-slate-300 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none"
                    placeholder="Enter the task or question for the agent..."
                  />
                </div>

                {/* Run Button */}
                <button
                  onClick={handleRunAgent}
                  disabled={!validation?.valid || !agentInput.trim() || isRunning}
                  className={`w-full flex items-center justify-center space-x-2 px-6 py-3 rounded-lg font-medium transition-colors ${
                    validation?.valid && agentInput.trim() && !isRunning
                      ? 'bg-blue-600 hover:bg-blue-700 text-white'
                      : 'bg-slate-700 text-slate-400 cursor-not-allowed'
                  }`}
                >
                  {isRunning ? (
                    <>
                      <Loader2 className="w-4 h-4 animate-spin" />
                      <span>Running...</span>
                    </>
                  ) : (
                    <>
                      <Play className="w-4 h-4" />
                      <span>Run Agent</span>
                    </>
                  )}
                </button>
              </div>
            </div>
          </div>
        )}

        {/* MCP Tools Tab */}
        {activeTab === 'mcp' && (
          <div className="h-[calc(100vh-12rem)]">
            <MCPToolsBrowser />
          </div>
        )}

        {/* Memory Tab */}
        {activeTab === 'memory' && (
          <div className="h-[calc(100vh-12rem)]">
            <MemoryViewer />
          </div>
        )}

        {/* Integrations Tab */}
        {activeTab === 'integrations' && (
          <div className="h-[calc(100vh-12rem)]">
            <PlatformIntegrations />
          </div>
        )}

        {/* Monitoring Tab */}
        {activeTab === 'monitoring' && (
          <div className="h-[calc(100vh-12rem)]">
            <SystemMonitoring />
          </div>
        )}

        {/* Templates Tab */}
        {activeTab === 'templates' && (
          <div className="h-[calc(100vh-12rem)]">
            <AgentTemplates onLoadTemplate={handleLoadTemplate} />
          </div>
        )}

        {/* Settings Tab */}
        {activeTab === 'settings' && (
          <div className="h-[calc(100vh-12rem)]">
            <Settings />
          </div>
        )}
      </main>
    </div>
  );
}

// Status Badge Component
function StatusBadge({ status }: { status: AgentStatus['status'] }) {
  const styles = {
    pending: 'bg-slate-600 text-slate-200',
    running: 'bg-blue-600 text-blue-100',
    completed: 'bg-green-600 text-green-100',
    failed: 'bg-red-600 text-red-100',
    stopped: 'bg-yellow-600 text-yellow-100',
  };

  return (
    <span className={`px-2 py-0.5 text-xs rounded-full ${styles[status]}`}>
      {status}
    </span>
  );
}

export default App;
