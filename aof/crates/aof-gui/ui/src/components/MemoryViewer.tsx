import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Brain, Search, Trash2, Download, Calendar, Clock,
  MessageSquare, AlertCircle, Loader2, Filter
} from 'lucide-react';
import { toast, invokeWithToast } from '../lib/toast';

interface MemoryEntry {
  id: string;
  timestamp: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  tokens?: number;
  metadata?: {
    model?: string;
    agent_id?: string;
    tool_calls?: number;
  };
}

interface MemoryStats {
  total_entries: number;
  total_tokens: number;
  conversation_start: string;
  last_updated: string;
  size_mb: number;
}

export function MemoryViewer() {
  const [entries, setEntries] = useState<MemoryEntry[]>([]);
  const [stats, setStats] = useState<MemoryStats | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [roleFilter, setRoleFilter] = useState<'all' | 'user' | 'assistant' | 'system'>('all');
  const [isLoading, setIsLoading] = useState(false);
  const [selectedEntry, setSelectedEntry] = useState<string | null>(null);

  useEffect(() => {
    loadMemory();
  }, []);

  const loadMemory = async () => {
    setIsLoading(true);
    try {
      const [memoryEntries, memoryStats] = await Promise.all([
        invoke<MemoryEntry[]>('memory_get_entries'),
        invoke<MemoryStats>('memory_get_stats'),
      ]);
      setEntries(memoryEntries);
      setStats(memoryStats);
    } catch (error) {
      toast.error('Failed to load memory', String(error));
      console.error('Failed to load memory:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleClearMemory = async () => {
    if (!confirm('Are you sure you want to clear all conversation memory? This cannot be undone.')) {
      return;
    }

    await invokeWithToast(
      'memory_clear',
      {},
      {
        loading: 'Clearing memory...',
        success: 'Memory cleared successfully',
        error: 'Failed to clear memory',
      }
    );
    await loadMemory();
  };

  const handleExportToJson = async () => {
    try {
      const json = await invoke<string>('memory_export_json');
      const blob = new Blob([json], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `aof-memory-${new Date().toISOString().split('T')[0]}.json`;
      a.click();
      URL.revokeObjectURL(url);
      toast.success('Memory exported to JSON');
    } catch (error) {
      toast.error('Failed to export memory', String(error));
    }
  };

  const handleExportToMarkdown = async () => {
    try {
      const markdown = await invoke<string>('memory_export_markdown');
      const blob = new Blob([markdown], { type: 'text/markdown' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `aof-memory-${new Date().toISOString().split('T')[0]}.md`;
      a.click();
      URL.revokeObjectURL(url);
      toast.success('Memory exported to Markdown');
    } catch (error) {
      toast.error('Failed to export memory', String(error));
    }
  };

  // Filter entries
  const filteredEntries = entries.filter(entry => {
    const matchesSearch = !searchQuery ||
      entry.content.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesRole = roleFilter === 'all' || entry.role === roleFilter;
    return matchesSearch && matchesRole;
  });

  const roleColors = {
    user: 'bg-blue-500/20 border-blue-500 text-blue-300',
    assistant: 'bg-green-500/20 border-green-500 text-green-300',
    system: 'bg-zinc-500/20 border-zinc-500 text-zinc-300',
  };

  return (
    <div className="flex h-full">
      {/* Sidebar - Stats & Controls */}
      <div className="w-80 border-r border-zinc-700 bg-zinc-900/50 flex flex-col">
        <div className="p-4 border-b border-zinc-700">
          <div className="flex items-center space-x-2 mb-3">
            <Brain className="w-6 h-6 text-sky-400" />
            <h3 className="text-lg font-semibold text-white">Memory & Context</h3>
          </div>
          <p className="text-sm text-zinc-400">
            View and manage conversation history
          </p>
        </div>

        {/* Statistics */}
        {stats && (
          <div className="p-4 border-b border-zinc-700 space-y-3">
            <h4 className="text-sm font-semibold text-white mb-2">Statistics</h4>

            <div className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span className="text-zinc-400">Total Entries</span>
                <span className="text-white font-medium">{stats.total_entries}</span>
              </div>

              <div className="flex items-center justify-between text-sm">
                <span className="text-zinc-400">Total Tokens</span>
                <span className="text-white font-medium">
                  {stats.total_tokens.toLocaleString()}
                </span>
              </div>

              <div className="flex items-center justify-between text-sm">
                <span className="text-zinc-400">Memory Size</span>
                <span className="text-white font-medium">
                  {stats.size_mb.toFixed(2)} MB
                </span>
              </div>
            </div>

            <div className="pt-2 border-t border-zinc-700 space-y-1">
              <div className="flex items-center space-x-2 text-xs text-zinc-500">
                <Calendar className="w-3 h-3" />
                <span>Started: {new Date(stats.conversation_start).toLocaleDateString()}</span>
              </div>
              <div className="flex items-center space-x-2 text-xs text-zinc-500">
                <Clock className="w-3 h-3" />
                <span>Updated: {new Date(stats.last_updated).toLocaleTimeString()}</span>
              </div>
            </div>
          </div>
        )}

        {/* Filters */}
        <div className="p-4 border-b border-zinc-700 space-y-3">
          <h4 className="text-sm font-semibold text-white mb-2">Filters</h4>

          <div>
            <label className="block text-xs text-zinc-400 mb-2">Role</label>
            <select
              value={roleFilter}
              onChange={(e) => setRoleFilter(e.target.value as any)}
              className="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-sky-400/60"
            >
              <option value="all">All Messages</option>
              <option value="user">User Only</option>
              <option value="assistant">Assistant Only</option>
              <option value="system">System Only</option>
            </select>
          </div>
        </div>

        {/* Actions */}
        <div className="mt-auto p-4 border-t border-zinc-700 space-y-2">
          <button
            onClick={handleExportToJson}
            className="w-full flex items-center justify-center space-x-2 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-white rounded-lg transition-colors"
          >
            <Download className="w-4 h-4" />
            <span>Export JSON</span>
          </button>

          <button
            onClick={handleExportToMarkdown}
            className="w-full flex items-center justify-center space-x-2 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-white rounded-lg transition-colors"
          >
            <Download className="w-4 h-4" />
            <span>Export Markdown</span>
          </button>

          <button
            onClick={handleClearMemory}
            className="w-full flex items-center justify-center space-x-2 px-4 py-2 bg-red-600/20 hover:bg-red-600/30 text-red-400 border border-red-600/50 rounded-lg transition-colors"
          >
            <Trash2 className="w-4 h-4" />
            <span>Clear Memory</span>
          </button>
        </div>
      </div>

      {/* Main Content - Timeline */}
      <div className="flex-1 flex flex-col">
        {/* Search Bar */}
        <div className="p-4 border-b border-zinc-700 bg-zinc-900/30">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-zinc-500" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search conversation history..."
              className="w-full pl-10 pr-4 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white placeholder-zinc-500 focus:outline-none focus:ring-2 focus:ring-sky-400/60"
            />
          </div>

          {searchQuery && (
            <p className="text-xs text-zinc-400 mt-2">
              Found {filteredEntries.length} of {entries.length} entries
            </p>
          )}
        </div>

        {/* Timeline */}
        <div className="flex-1 overflow-y-auto p-6">
          {isLoading ? (
            <div className="flex items-center justify-center h-full">
              <Loader2 className="w-8 h-8 text-sky-400 animate-spin" />
            </div>
          ) : filteredEntries.length === 0 ? (
            <div className="flex items-center justify-center h-full text-zinc-500">
              <div className="text-center">
                {searchQuery || roleFilter !== 'all' ? (
                  <>
                    <Filter className="w-16 h-16 mx-auto mb-4 opacity-50" />
                    <p className="text-lg mb-2">No matching entries</p>
                    <p className="text-sm">Try adjusting your search or filters</p>
                  </>
                ) : (
                  <>
                    <MessageSquare className="w-16 h-16 mx-auto mb-4 opacity-50" />
                    <p className="text-lg mb-2">No conversation history</p>
                    <p className="text-sm">Memory entries will appear here after running agents</p>
                  </>
                )}
              </div>
            </div>
          ) : (
            <div className="space-y-4 max-w-4xl mx-auto">
              {filteredEntries.map((entry) => (
                <div
                  key={entry.id}
                  onClick={() => setSelectedEntry(entry.id === selectedEntry ? null : entry.id)}
                  className={`border rounded-lg overflow-hidden transition-all cursor-pointer ${
                    roleColors[entry.role]
                  } ${
                    selectedEntry === entry.id
                      ? 'ring-2 ring-sky-400/60'
                      : ''
                  }`}
                >
                  {/* Header */}
                  <div className="px-4 py-2 bg-zinc-900/50 border-b border-zinc-800 flex items-center justify-between">
                    <div className="flex items-center space-x-3">
                      <span className="text-xs font-semibold uppercase tracking-wide">
                        {entry.role}
                      </span>
                      <span className="text-xs text-zinc-500">
                        {new Date(entry.timestamp).toLocaleString()}
                      </span>
                      {entry.tokens && (
                        <span className="text-xs text-zinc-500">
                          {entry.tokens} tokens
                        </span>
                      )}
                    </div>

                    {entry.metadata && (
                      <div className="flex items-center space-x-2 text-xs text-zinc-500">
                        {entry.metadata.model && (
                          <span>{entry.metadata.model}</span>
                        )}
                        {entry.metadata.tool_calls && entry.metadata.tool_calls > 0 && (
                          <span>🔧 {entry.metadata.tool_calls} tools</span>
                        )}
                      </div>
                    )}
                  </div>

                  {/* Content */}
                  <div className="p-4">
                    <div className={`text-sm text-white ${
                      selectedEntry === entry.id ? '' : 'line-clamp-3'
                    }`}>
                      {entry.content}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
