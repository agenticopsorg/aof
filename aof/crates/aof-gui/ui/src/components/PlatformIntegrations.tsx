import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  MessageCircle, Send, Smartphone, Globe, Check,
  AlertCircle, Loader2, Eye, EyeOff, RefreshCw, Activity
} from 'lucide-react';
import { toast, invokeWithToast } from '../lib/toast';

interface Integration {
  id: string;
  platform: 'slack' | 'telegram' | 'whatsapp';
  name: string;
  enabled: boolean;
  configured: boolean;
  status: 'active' | 'inactive' | 'error';
  config?: {
    webhook_url?: string;
    bot_token?: string;
    chat_id?: string;
    phone_number?: string;
    api_key?: string;
  };
  stats?: {
    messages_sent: number;
    messages_received: number;
    last_message: string;
  };
}

interface IntegrationLog {
  timestamp: string;
  platform: string;
  type: 'sent' | 'received' | 'error';
  message: string;
  details?: string;
}

export function PlatformIntegrations() {
  const [integrations, setIntegrations] = useState<Integration[]>([]);
  const [selectedIntegration, setSelectedIntegration] = useState<string | null>(null);
  const [logs, setLogs] = useState<IntegrationLog[]>([]);
  const [testMessage, setTestMessage] = useState('');
  const [isTesting, setIsTesting] = useState(false);
  const [showSecrets, setShowSecrets] = useState<Record<string, boolean>>({});
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    loadIntegrations();
    loadLogs();
  }, []);

  const loadIntegrations = async () => {
    setIsLoading(true);
    try {
      const data = await invoke<Integration[]>('integrations_list');
      setIntegrations(data);
    } catch (error) {
      toast.error('Failed to load integrations', String(error));
      console.error('Failed to load integrations:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const loadLogs = async () => {
    try {
      const data = await invoke<IntegrationLog[]>('integrations_get_logs', { limit: 50 });
      setLogs(data);
    } catch (error) {
      console.error('Failed to load logs:', error);
    }
  };

  const handleToggleIntegration = async (id: string, enabled: boolean) => {
    await invokeWithToast(
      'integrations_toggle',
      { id, enabled },
      {
        loading: enabled ? 'Enabling integration...' : 'Disabling integration...',
        success: enabled ? 'Integration enabled' : 'Integration disabled',
        error: 'Failed to toggle integration',
      }
    );
    await loadIntegrations();
  };

  const handleTestIntegration = async (id: string) => {
    if (!testMessage.trim()) {
      toast.error('Please enter a test message');
      return;
    }

    setIsTesting(true);
    try {
      await invoke('integrations_test', { id, message: testMessage });
      toast.success('Test message sent successfully');
      setTestMessage('');
      await loadLogs();
    } catch (error) {
      toast.error('Failed to send test message', String(error));
    } finally {
      setIsTesting(false);
    }
  };

  const selectedIntegrationData = integrations.find(i => i.id === selectedIntegration);

  const platformIcons = {
    slack: MessageCircle,
    telegram: Send,
    whatsapp: Smartphone,
  };

  const platformColors = {
    slack: 'from-purple-600/20 to-pink-600/20 border-purple-500/50',
    telegram: 'from-blue-600/20 to-cyan-600/20 border-blue-500/50',
    whatsapp: 'from-green-600/20 to-emerald-600/20 border-green-500/50',
  };

  const statusColors = {
    active: 'bg-green-500',
    inactive: 'bg-zinc-500',
    error: 'bg-red-500',
  };

  return (
    <div className="flex h-full">
      {/* Sidebar - Integration Cards */}
      <div className="w-96 border-r border-zinc-700 bg-zinc-900/50 flex flex-col">
        <div className="p-4 border-b border-zinc-700">
          <h3 className="text-lg font-semibold text-white mb-1">Platform Integrations</h3>
          <p className="text-sm text-zinc-400">
            Connect AOF to messaging platforms
          </p>
        </div>

        <div className="flex-1 overflow-y-auto p-4 space-y-3">
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="w-8 h-8 text-sky-400 animate-spin" />
            </div>
          ) : integrations.length === 0 ? (
            <div className="text-center py-8 text-zinc-500">
              <Globe className="w-12 h-12 mx-auto mb-2 opacity-50" />
              <p className="text-sm">No integrations configured</p>
            </div>
          ) : (
            integrations.map((integration) => {
              const Icon = platformIcons[integration.platform];
              return (
                <div
                  key={integration.id}
                  onClick={() => setSelectedIntegration(integration.id)}
                  className={`p-4 rounded-lg border-2 bg-gradient-to-br cursor-pointer transition-all ${
                    platformColors[integration.platform]
                  } ${
                    selectedIntegration === integration.id
                      ? 'ring-2 ring-sky-400/60'
                      : 'hover:border-zinc-600'
                  }`}
                >
                  <div className="flex items-start justify-between mb-3">
                    <div className="flex items-center space-x-3">
                      <Icon className="w-6 h-6 text-white" />
                      <div>
                        <h4 className="text-white font-semibold capitalize">
                          {integration.platform}
                        </h4>
                        <p className="text-xs text-zinc-400">{integration.name}</p>
                      </div>
                    </div>
                    <div className={`w-2 h-2 rounded-full ${statusColors[integration.status]}`} />
                  </div>

                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-4 text-xs text-zinc-400">
                      {integration.stats && (
                        <>
                          <span>↑ {integration.stats.messages_sent}</span>
                          <span>↓ {integration.stats.messages_received}</span>
                        </>
                      )}
                    </div>

                    <div className="flex items-center space-x-2">
                      {integration.configured ? (
                        <Check className="w-4 h-4 text-green-400" />
                      ) : (
                        <AlertCircle className="w-4 h-4 text-yellow-400" />
                      )}
                    </div>
                  </div>
                </div>
              );
            })
          )}
        </div>

        {/* Refresh Button */}
        <div className="p-4 border-t border-zinc-700">
          <button
            onClick={loadIntegrations}
            className="w-full flex items-center justify-center space-x-2 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-white rounded-lg transition-colors"
          >
            <RefreshCw className="w-4 h-4" />
            <span>Refresh Status</span>
          </button>
        </div>
      </div>

      {/* Main Content - Configuration & Testing */}
      <div className="flex-1 flex flex-col">
        {!selectedIntegrationData ? (
          <div className="flex items-center justify-center h-full text-zinc-500">
            <div className="text-center">
              <MessageCircle className="w-16 h-16 mx-auto mb-4 opacity-50" />
              <p className="text-lg">Select an integration to configure</p>
            </div>
          </div>
        ) : (
          <>
            {/* Configuration Panel */}
            <div className="p-6 border-b border-zinc-700 bg-zinc-900/30">
              <div className="flex items-center justify-between mb-6">
                <div className="flex items-center space-x-3">
                  {(() => {
                    const Icon = platformIcons[selectedIntegrationData.platform];
                    return <Icon className="w-8 h-8 text-white" />;
                  })()}
                  <div>
                    <h2 className="text-2xl font-bold text-white capitalize">
                      {selectedIntegrationData.platform} Integration
                    </h2>
                    <p className="text-zinc-400">{selectedIntegrationData.name}</p>
                  </div>
                </div>

                <label className="flex items-center space-x-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={selectedIntegrationData.enabled}
                    onChange={(e) => handleToggleIntegration(
                      selectedIntegrationData.id,
                      e.target.checked
                    )}
                    className="w-5 h-5 rounded bg-zinc-800 border-zinc-700"
                  />
                  <span className="text-white font-medium">Enabled</span>
                </label>
              </div>

              {/* Configuration Form */}
              <div className="space-y-4">
                {selectedIntegrationData.platform === 'slack' && (
                  <>
                    <div>
                      <label className="block text-sm font-medium text-zinc-300 mb-2">
                        Webhook URL
                      </label>
                      <div className="relative">
                        <input
                          type={showSecrets['slack_webhook'] ? 'text' : 'password'}
                          defaultValue={selectedIntegrationData.config?.webhook_url || ''}
                          placeholder="https://hooks.slack.com/services/..."
                          className="w-full px-4 py-2 pr-10 bg-zinc-800 border border-zinc-700 rounded-lg text-white placeholder-zinc-500 focus:outline-none focus:ring-2 focus:ring-sky-400/60"
                        />
                        <button
                          onClick={() => setShowSecrets({
                            ...showSecrets,
                            slack_webhook: !showSecrets['slack_webhook']
                          })}
                          className="absolute right-2 top-1/2 -translate-y-1/2 p-1 hover:bg-zinc-700 rounded"
                        >
                          {showSecrets['slack_webhook'] ? (
                            <EyeOff className="w-4 h-4 text-zinc-400" />
                          ) : (
                            <Eye className="w-4 h-4 text-zinc-400" />
                          )}
                        </button>
                      </div>
                    </div>
                    <div>
                      <label className="block text-sm font-medium text-zinc-300 mb-2">
                        Bot Token (Optional)
                      </label>
                      <input
                        type={showSecrets['slack_token'] ? 'text' : 'password'}
                        defaultValue={selectedIntegrationData.config?.bot_token || ''}
                        placeholder="xoxb-..."
                        className="w-full px-4 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white placeholder-zinc-500 focus:outline-none focus:ring-2 focus:ring-sky-400/60"
                      />
                    </div>
                  </>
                )}

                {selectedIntegrationData.platform === 'telegram' && (
                  <>
                    <div>
                      <label className="block text-sm font-medium text-zinc-300 mb-2">
                        Bot Token
                      </label>
                      <input
                        type={showSecrets['telegram_token'] ? 'text' : 'password'}
                        defaultValue={selectedIntegrationData.config?.bot_token || ''}
                        placeholder="123456:ABC-DEF..."
                        className="w-full px-4 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white placeholder-zinc-500 focus:outline-none focus:ring-2 focus:ring-sky-400/60"
                      />
                    </div>
                    <div>
                      <label className="block text-sm font-medium text-zinc-300 mb-2">
                        Chat ID
                      </label>
                      <input
                        type="text"
                        defaultValue={selectedIntegrationData.config?.chat_id || ''}
                        placeholder="-1001234567890"
                        className="w-full px-4 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white placeholder-zinc-500 focus:outline-none focus:ring-2 focus:ring-sky-400/60"
                      />
                    </div>
                  </>
                )}

                {selectedIntegrationData.platform === 'whatsapp' && (
                  <>
                    <div>
                      <label className="block text-sm font-medium text-zinc-300 mb-2">
                        Phone Number
                      </label>
                      <input
                        type="text"
                        defaultValue={selectedIntegrationData.config?.phone_number || ''}
                        placeholder="+1234567890"
                        className="w-full px-4 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white placeholder-zinc-500 focus:outline-none focus:ring-2 focus:ring-sky-400/60"
                      />
                    </div>
                    <div>
                      <label className="block text-sm font-medium text-zinc-300 mb-2">
                        API Key
                      </label>
                      <input
                        type={showSecrets['whatsapp_key'] ? 'text' : 'password'}
                        defaultValue={selectedIntegrationData.config?.api_key || ''}
                        placeholder="Your WhatsApp API key"
                        className="w-full px-4 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white placeholder-zinc-500 focus:outline-none focus:ring-2 focus:ring-sky-400/60"
                      />
                    </div>
                  </>
                )}

                <button
                  className="w-full px-4 py-2 bg-sky-400/60 hover:bg-sky-400/80 text-white rounded-lg transition-colors"
                >
                  Save Configuration
                </button>
              </div>

              {/* Test Message */}
              <div className="mt-6 pt-6 border-t border-zinc-700">
                <h3 className="text-lg font-semibold text-white mb-3">Test Integration</h3>
                <div className="flex space-x-2">
                  <input
                    type="text"
                    value={testMessage}
                    onChange={(e) => setTestMessage(e.target.value)}
                    placeholder="Enter test message..."
                    className="flex-1 px-4 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white placeholder-zinc-500 focus:outline-none focus:ring-2 focus:ring-sky-400/60"
                  />
                  <button
                    onClick={() => handleTestIntegration(selectedIntegrationData.id)}
                    disabled={isTesting || !selectedIntegrationData.enabled}
                    className="flex items-center space-x-2 px-6 py-2 bg-sky-400/60 hover:bg-sky-400/80 disabled:opacity-50 text-white rounded-lg transition-colors"
                  >
                    {isTesting ? (
                      <>
                        <Loader2 className="w-4 h-4 animate-spin" />
                        <span>Sending...</span>
                      </>
                    ) : (
                      <>
                        <Send className="w-4 h-4" />
                        <span>Send Test</span>
                      </>
                    )}
                  </button>
                </div>
              </div>
            </div>

            {/* Activity Logs */}
            <div className="flex-1 overflow-y-auto p-6">
              <div className="flex items-center space-x-2 mb-4">
                <Activity className="w-5 h-5 text-white" />
                <h3 className="text-lg font-semibold text-white">Activity Log</h3>
              </div>

              {logs.length === 0 ? (
                <div className="text-center py-8 text-zinc-500">
                  <p className="text-sm">No activity yet</p>
                </div>
              ) : (
                <div className="space-y-2">
                  {logs.map((log, index) => (
                    <div
                      key={index}
                      className={`p-3 rounded-lg border ${
                        log.type === 'error'
                          ? 'bg-red-900/20 border-red-700'
                          : log.type === 'sent'
                          ? 'bg-blue-900/20 border-blue-700'
                          : 'bg-green-900/20 border-green-700'
                      }`}
                    >
                      <div className="flex items-center justify-between mb-1">
                        <span className="text-xs font-semibold uppercase text-white">
                          {log.platform} • {log.type}
                        </span>
                        <span className="text-xs text-zinc-500">
                          {new Date(log.timestamp).toLocaleTimeString()}
                        </span>
                      </div>
                      <p className="text-sm text-zinc-300">{log.message}</p>
                      {log.details && (
                        <p className="text-xs text-zinc-500 mt-1">{log.details}</p>
                      )}
                    </div>
                  ))}
                </div>
              )}
            </div>
          </>
        )}
      </div>
    </div>
  );
}
