import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Key, Palette, Activity,
  Save, RotateCcw, Download, Upload, CheckCircle, XCircle, Loader2
} from 'lucide-react';
import { invokeWithToast } from '../lib/toast';

// Types matching Rust backend
interface ProviderConfig {
  provider: string;
  api_key?: string;
  base_url?: string;
  default_model: string;
}

interface AppSettings {
  theme: string;
  default_provider: string;
  default_temperature: number;
  default_max_tokens: number;
  auto_save: boolean;
  log_level: string;
  providers: ProviderConfig[];
}

interface ConnectionStatus {
  provider: string;
  status: 'idle' | 'testing' | 'success' | 'error';
  message?: string;
}

// Main colors: white text with bg-zinc-900, buttons: bg-sky-400/60
export function Settings() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [activeSection, setActiveSection] = useState<'general' | 'providers' | 'advanced'>('providers');
  const [connectionStatuses, setConnectionStatuses] = useState<ConnectionStatus[]>([]);
  const [models, setModels] = useState<Record<string, string[]>>({});

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const data = await invoke<AppSettings>('settings_get');
      setSettings(data);

      // Initialize connection statuses
      setConnectionStatuses(data.providers.map(p => ({
        provider: p.provider,
        status: 'idle'
      })));

      // Load models for each provider
      for (const provider of data.providers) {
        const providerModels = await invoke<string[]>('provider_list_models', {
          provider: provider.provider
        });
        setModels(prev => ({ ...prev, [provider.provider]: providerModels }));
      }
    } catch (error) {
      console.error('Failed to load settings:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    if (!settings) return;

    setSaving(true);
    try {
      await invokeWithToast('settings_update', { settings }, {
        loading: 'Saving settings...',
        success: 'Settings saved successfully!',
        error: 'Failed to save settings',
      });
    } catch (error) {
      console.error('Failed to save settings:', error);
    } finally {
      setSaving(false);
    }
  };

  const handleReset = async () => {
    if (!confirm('Reset all settings to defaults?')) return;

    try {
      const defaults = await invokeWithToast<AppSettings>('settings_reset', undefined, {
        loading: 'Resetting settings...',
        success: 'Settings reset to defaults',
        error: 'Failed to reset settings',
      });
      setSettings(defaults);
    } catch (error) {
      console.error('Failed to reset settings:', error);
    }
  };

  const handleTestConnection = async (provider: string, apiKey: string, baseUrl?: string) => {
    setConnectionStatuses(prev => prev.map(s =>
      s.provider === provider ? { ...s, status: 'testing', message: undefined } : s
    ));

    try {
      const message = await invoke<string>('provider_test_connection', {
        provider,
        apiKey,
        baseUrl
      });

      setConnectionStatuses(prev => prev.map(s =>
        s.provider === provider ? { provider, status: 'success', message } : s
      ));
    } catch (error) {
      setConnectionStatuses(prev => prev.map(s =>
        s.provider === provider ? {
          provider,
          status: 'error',
          message: String(error)
        } : s
      ));
    }
  };

  const updateProvider = (provider: string, updates: Partial<ProviderConfig>) => {
    if (!settings) return;

    setSettings({
      ...settings,
      providers: settings.providers.map(p =>
        p.provider === provider ? { ...p, ...updates } : p
      )
    });
  };

  const handleExport = async () => {
    try {
      const json = await invoke<string>('settings_export');
      const blob = new Blob([json], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'aof-settings.json';
      a.click();
      URL.revokeObjectURL(url);
    } catch (error) {
      console.error('Failed to export settings:', error);
    }
  };

  const handleImport = () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = 'application/json';
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;

      const text = await file.text();
      try {
        await invoke('settings_import', { jsonContent: text });
        await loadSettings();
        alert('Settings imported successfully!');
      } catch (error) {
        alert(`Failed to import settings: ${error}`);
      }
    };
    input.click();
  };

  if (loading || !settings) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="w-8 h-8 text-sky-400/60 animate-spin" />
      </div>
    );
  }

  const getConnectionStatus = (provider: string) => {
    return connectionStatuses.find(s => s.provider === provider);
  };

  return (
    <div className="h-full flex">
      {/* Sidebar */}
      <div className="w-64 border-r border-zinc-700 bg-zinc-900/50">
        <div className="p-4 space-y-1">
          <button
            onClick={() => setActiveSection('providers')}
            className={`w-full flex items-center space-x-3 px-4 py-3 rounded-lg transition-colors ${
              activeSection === 'providers'
                ? 'bg-sky-400/60 text-white'
                : 'hover:bg-zinc-800 text-zinc-400'
            }`}
          >
            <Key className="w-5 h-5" />
            <span className="font-medium">LLM Providers</span>
          </button>

          <button
            onClick={() => setActiveSection('general')}
            className={`w-full flex items-center space-x-3 px-4 py-3 rounded-lg transition-colors ${
              activeSection === 'general'
                ? 'bg-sky-400/60 text-white'
                : 'hover:bg-zinc-800 text-zinc-400'
            }`}
          >
            <Palette className="w-5 h-5" />
            <span className="font-medium">General</span>
          </button>

          <button
            onClick={() => setActiveSection('advanced')}
            className={`w-full flex items-center space-x-3 px-4 py-3 rounded-lg transition-colors ${
              activeSection === 'advanced'
                ? 'bg-sky-400/60 text-white'
                : 'hover:bg-zinc-800 text-zinc-400'
            }`}
          >
            <Activity className="w-5 h-5" />
            <span className="font-medium">Advanced</span>
          </button>
        </div>

        {/* Action Buttons */}
        <div className="absolute bottom-0 left-0 right-0 p-4 border-t border-zinc-700 bg-zinc-900/80 space-y-2">
          <button
            onClick={handleSave}
            disabled={saving}
            className="w-full flex items-center justify-center space-x-2 px-4 py-2 bg-sky-400/60 hover:bg-sky-400/80 text-white rounded-lg transition-colors disabled:opacity-50"
          >
            {saving ? (
              <>
                <Loader2 className="w-4 h-4 animate-spin" />
                <span>Saving...</span>
              </>
            ) : (
              <>
                <Save className="w-4 h-4" />
                <span>Save Settings</span>
              </>
            )}
          </button>

          <button
            onClick={handleReset}
            className="w-full flex items-center justify-center space-x-2 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded-lg transition-colors"
          >
            <RotateCcw className="w-4 h-4" />
            <span>Reset</span>
          </button>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 overflow-y-auto p-6">
        {/* Providers Section */}
        {activeSection === 'providers' && (
          <div className="space-y-6">
            <div>
              <h2 className="text-2xl font-bold text-white mb-2">LLM Providers</h2>
              <p className="text-zinc-400">Configure API keys and settings for each provider</p>
            </div>

            {settings.providers.map(provider => {
              const status = getConnectionStatus(provider.provider);
              const providerModels = models[provider.provider] || [];

              return (
                <div key={provider.provider} className="bg-zinc-800/50 border border-zinc-700 rounded-lg p-6">
                  <div className="flex items-center justify-between mb-4">
                    <h3 className="text-lg font-semibold text-white capitalize">
                      {provider.provider}
                    </h3>

                    {status && (
                      <div className="flex items-center space-x-2">
                        {status.status === 'testing' && (
                          <Loader2 className="w-4 h-4 text-sky-400/60 animate-spin" />
                        )}
                        {status.status === 'success' && (
                          <CheckCircle className="w-4 h-4 text-green-400" />
                        )}
                        {status.status === 'error' && (
                          <XCircle className="w-4 h-4 text-red-400" />
                        )}
                        <span className={`text-sm ${
                          status.status === 'success' ? 'text-green-400' :
                          status.status === 'error' ? 'text-red-400' :
                          'text-zinc-400'
                        }`}>
                          {status.message}
                        </span>
                      </div>
                    )}
                  </div>

                  <div className="space-y-4">
                    {/* API Key */}
                    {provider.provider !== 'ollama' && (
                      <div>
                        <label className="block text-sm font-medium text-zinc-300 mb-2">
                          API Key
                        </label>
                        <input
                          type="password"
                          value={provider.api_key || ''}
                          onChange={(e) => updateProvider(provider.provider, { api_key: e.target.value })}
                          className="w-full px-4 py-2 bg-zinc-900 border border-zinc-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-sky-400/60"
                          placeholder={`Enter ${provider.provider} API key`}
                        />
                      </div>
                    )}

                    {/* Base URL (for Ollama/custom endpoints) */}
                    {(provider.provider === 'ollama' || provider.provider === 'openai') && (
                      <div>
                        <label className="block text-sm font-medium text-zinc-300 mb-2">
                          Base URL (Optional)
                        </label>
                        <input
                          type="text"
                          value={provider.base_url || ''}
                          onChange={(e) => updateProvider(provider.provider, { base_url: e.target.value })}
                          className="w-full px-4 py-2 bg-zinc-900 border border-zinc-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-sky-400/60"
                          placeholder={provider.provider === 'ollama' ? 'http://localhost:11434' : 'Custom API endpoint'}
                        />
                      </div>
                    )}

                    {/* Default Model */}
                    <div>
                      <label className="block text-sm font-medium text-zinc-300 mb-2">
                        Default Model
                      </label>
                      <select
                        value={provider.default_model}
                        onChange={(e) => updateProvider(provider.provider, { default_model: e.target.value })}
                        className="w-full px-4 py-2 bg-zinc-900 border border-zinc-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-sky-400/60"
                      >
                        {providerModels.map(model => (
                          <option key={model} value={model}>{model}</option>
                        ))}
                      </select>
                    </div>

                    {/* Test Connection */}
                    <button
                      onClick={() => handleTestConnection(
                        provider.provider,
                        provider.api_key || '',
                        provider.base_url
                      )}
                      disabled={!provider.api_key && provider.provider !== 'ollama'}
                      className="px-4 py-2 bg-zinc-700 hover:bg-zinc-600 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      Test Connection
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        )}

        {/* General Section */}
        {activeSection === 'general' && (
          <div className="space-y-6">
            <div>
              <h2 className="text-2xl font-bold text-white mb-2">General Settings</h2>
              <p className="text-zinc-400">Application preferences and behavior</p>
            </div>

            <div className="bg-zinc-800/50 border border-zinc-700 rounded-lg p-6 space-y-4">
              {/* Theme */}
              <div>
                <label className="block text-sm font-medium text-zinc-300 mb-2">
                  Theme
                </label>
                <select
                  value={settings.theme}
                  onChange={(e) => setSettings({ ...settings, theme: e.target.value })}
                  className="w-full px-4 py-2 bg-zinc-900 border border-zinc-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-sky-400/60"
                >
                  <option value="dark">Dark</option>
                  <option value="light">Light</option>
                  <option value="auto">Auto</option>
                </select>
              </div>

              {/* Auto Save */}
              <div className="flex items-center justify-between">
                <div>
                  <label className="block text-sm font-medium text-zinc-300 mb-1">
                    Auto Save Configurations
                  </label>
                  <p className="text-sm text-zinc-500">Automatically save agent configurations</p>
                </div>
                <input
                  type="checkbox"
                  checked={settings.auto_save}
                  onChange={(e) => setSettings({ ...settings, auto_save: e.target.checked })}
                  className="w-5 h-5 rounded bg-zinc-900 border-zinc-700"
                />
              </div>

              {/* Default Temperature */}
              <div>
                <label className="block text-sm font-medium text-zinc-300 mb-2">
                  Default Temperature: {settings.default_temperature}
                </label>
                <input
                  type="range"
                  min="0"
                  max="2"
                  step="0.1"
                  value={settings.default_temperature}
                  onChange={(e) => setSettings({ ...settings, default_temperature: parseFloat(e.target.value) })}
                  className="w-full"
                />
              </div>

              {/* Default Max Tokens */}
              <div>
                <label className="block text-sm font-medium text-zinc-300 mb-2">
                  Default Max Tokens
                </label>
                <input
                  type="number"
                  value={settings.default_max_tokens}
                  onChange={(e) => setSettings({ ...settings, default_max_tokens: parseInt(e.target.value) })}
                  className="w-full px-4 py-2 bg-zinc-900 border border-zinc-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-sky-400/60"
                />
              </div>
            </div>
          </div>
        )}

        {/* Advanced Section */}
        {activeSection === 'advanced' && (
          <div className="space-y-6">
            <div>
              <h2 className="text-2xl font-bold text-white mb-2">Advanced Settings</h2>
              <p className="text-zinc-400">Import, export, and logging configuration</p>
            </div>

            <div className="bg-zinc-800/50 border border-zinc-700 rounded-lg p-6 space-y-4">
              {/* Log Level */}
              <div>
                <label className="block text-sm font-medium text-zinc-300 mb-2">
                  Log Level
                </label>
                <select
                  value={settings.log_level}
                  onChange={(e) => setSettings({ ...settings, log_level: e.target.value })}
                  className="w-full px-4 py-2 bg-zinc-900 border border-zinc-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-sky-400/60"
                >
                  <option value="error">Error</option>
                  <option value="warn">Warning</option>
                  <option value="info">Info</option>
                  <option value="debug">Debug</option>
                  <option value="trace">Trace</option>
                </select>
              </div>

              {/* Import/Export */}
              <div className="pt-4 border-t border-zinc-700">
                <h3 className="text-sm font-medium text-zinc-300 mb-3">Import / Export</h3>
                <div className="flex space-x-3">
                  <button
                    onClick={handleExport}
                    className="flex-1 flex items-center justify-center space-x-2 px-4 py-2 bg-zinc-700 hover:bg-zinc-600 text-white rounded-lg transition-colors"
                  >
                    <Download className="w-4 h-4" />
                    <span>Export Settings</span>
                  </button>

                  <button
                    onClick={handleImport}
                    className="flex-1 flex items-center justify-center space-x-2 px-4 py-2 bg-zinc-700 hover:bg-zinc-600 text-white rounded-lg transition-colors"
                  >
                    <Upload className="w-4 h-4" />
                    <span>Import Settings</span>
                  </button>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
