import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Activity, DollarSign, Zap, TrendingUp, Download,
  Clock, CheckCircle, XCircle, Loader2, BarChart3
} from 'lucide-react';
import { toast } from '../lib/toast';

interface SystemMetrics {
  agents: {
    total: number;
    running: number;
    completed: number;
    failed: number;
  };
  tokens: {
    total_input: number;
    total_output: number;
    total: number;
  };
  costs: {
    total: number;
    by_provider: Record<string, number>;
    by_model: Record<string, number>;
  };
  performance: {
    avg_response_time_ms: number;
    success_rate: number;
    total_executions: number;
  };
  timeline: Array<{
    timestamp: string;
    tokens: number;
    cost: number;
    executions: number;
  }>;
}

interface MetricCard {
  title: string;
  value: string | number;
  subtitle?: string;
  icon: typeof Activity;
  color: string;
  trend?: {
    value: number;
    direction: 'up' | 'down';
  };
}

export function SystemMonitoring() {
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null);
  const [timeRange, setTimeRange] = useState<'24h' | '7d' | '30d'>('24h');
  const [isLoading, setIsLoading] = useState(false);
  const [autoRefresh, setAutoRefresh] = useState(false);

  useEffect(() => {
    loadMetrics();
  }, [timeRange]);

  useEffect(() => {
    if (!autoRefresh) return;

    const interval = setInterval(() => {
      loadMetrics();
    }, 10000); // Refresh every 10 seconds

    return () => clearInterval(interval);
  }, [autoRefresh, timeRange]);

  const loadMetrics = async () => {
    setIsLoading(true);
    try {
      const data = await invoke<SystemMetrics>('monitoring_get_metrics', { timeRange });
      setMetrics(data);
    } catch (error) {
      toast.error('Failed to load metrics', String(error));
      console.error('Failed to load metrics:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleExportCSV = async () => {
    try {
      const csv = await invoke<string>('monitoring_export_csv', { timeRange });
      const blob = new Blob([csv], { type: 'text/csv' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `aof-metrics-${timeRange}-${new Date().toISOString().split('T')[0]}.csv`;
      a.click();
      URL.revokeObjectURL(url);
      toast.success('Metrics exported to CSV');
    } catch (error) {
      toast.error('Failed to export metrics', String(error));
    }
  };

  if (!metrics) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="w-8 h-8 text-sky-400 animate-spin" />
      </div>
    );
  }

  const metricCards: MetricCard[] = [
    {
      title: 'Active Agents',
      value: metrics.agents.running,
      subtitle: `${metrics.agents.total} total`,
      icon: Activity,
      color: 'from-blue-600/20 to-cyan-600/20 border-blue-500/50',
    },
    {
      title: 'Total Tokens',
      value: metrics.tokens.total.toLocaleString(),
      subtitle: `${(metrics.tokens.total / 1000000).toFixed(2)}M`,
      icon: Zap,
      color: 'from-yellow-600/20 to-orange-600/20 border-yellow-500/50',
    },
    {
      title: 'Total Cost',
      value: `$${metrics.costs.total.toFixed(2)}`,
      subtitle: timeRange,
      icon: DollarSign,
      color: 'from-green-600/20 to-emerald-600/20 border-green-500/50',
    },
    {
      title: 'Success Rate',
      value: `${(metrics.performance.success_rate * 100).toFixed(1)}%`,
      subtitle: `${metrics.agents.completed}/${metrics.performance.total_executions}`,
      icon: CheckCircle,
      color: 'from-purple-600/20 to-pink-600/20 border-purple-500/50',
    },
  ];

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="p-6 border-b border-zinc-700 bg-zinc-900/50">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center space-x-3">
            <BarChart3 className="w-8 h-8 text-sky-400" />
            <div>
              <h2 className="text-2xl font-bold text-white">System Monitoring</h2>
              <p className="text-zinc-400">Usage metrics and analytics</p>
            </div>
          </div>

          <div className="flex items-center space-x-3">
            <label className="flex items-center space-x-2 cursor-pointer">
              <input
                type="checkbox"
                checked={autoRefresh}
                onChange={(e) => setAutoRefresh(e.target.checked)}
                className="w-4 h-4 rounded bg-zinc-800 border-zinc-700"
              />
              <span className="text-sm text-zinc-300">Auto-refresh</span>
            </label>

            <button
              onClick={loadMetrics}
              disabled={isLoading}
              className="p-2 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors"
            >
              <Loader2 className={`w-4 h-4 text-white ${isLoading ? 'animate-spin' : ''}`} />
            </button>

            <button
              onClick={handleExportCSV}
              className="flex items-center space-x-2 px-4 py-2 bg-sky-400/60 hover:bg-sky-400/80 text-white rounded-lg transition-colors"
            >
              <Download className="w-4 h-4" />
              <span>Export CSV</span>
            </button>
          </div>
        </div>

        {/* Time Range Selector */}
        <div className="flex space-x-2">
          {(['24h', '7d', '30d'] as const).map((range) => (
            <button
              key={range}
              onClick={() => setTimeRange(range)}
              className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                timeRange === range
                  ? 'bg-sky-400/60 text-white'
                  : 'bg-zinc-800 text-zinc-400 hover:bg-zinc-700'
              }`}
            >
              Last {range}
            </button>
          ))}
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 overflow-y-auto p-6">
        {/* Metric Cards */}
        <div className="grid grid-cols-4 gap-4 mb-8">
          {metricCards.map((card) => (
            <div
              key={card.title}
              className={`p-4 rounded-lg border-2 bg-gradient-to-br ${card.color}`}
            >
              <div className="flex items-start justify-between mb-2">
                <card.icon className="w-6 h-6 text-white" />
                {card.trend && (
                  <div className={`flex items-center text-xs ${
                    card.trend.direction === 'up' ? 'text-green-400' : 'text-red-400'
                  }`}>
                    <TrendingUp className={`w-3 h-3 mr-1 ${
                      card.trend.direction === 'down' ? 'rotate-180' : ''
                    }`} />
                    {card.trend.value}%
                  </div>
                )}
              </div>
              <div className="text-2xl font-bold text-white mb-1">{card.value}</div>
              <div className="text-xs text-zinc-400">{card.title}</div>
              {card.subtitle && (
                <div className="text-xs text-zinc-500 mt-1">{card.subtitle}</div>
              )}
            </div>
          ))}
        </div>

        {/* Token Distribution */}
        <div className="grid grid-cols-2 gap-6 mb-8">
          <div className="bg-zinc-800/50 backdrop-blur rounded-lg border border-zinc-700 p-6">
            <h3 className="text-lg font-semibold text-white mb-4">Token Usage</h3>
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-zinc-400">Input Tokens</span>
                <span className="text-white font-medium">
                  {metrics.tokens.total_input.toLocaleString()}
                </span>
              </div>
              <div className="w-full bg-zinc-900 rounded-full h-2">
                <div
                  className="bg-blue-500 h-2 rounded-full"
                  style={{
                    width: `${(metrics.tokens.total_input / metrics.tokens.total) * 100}%`,
                  }}
                />
              </div>

              <div className="flex items-center justify-between">
                <span className="text-zinc-400">Output Tokens</span>
                <span className="text-white font-medium">
                  {metrics.tokens.total_output.toLocaleString()}
                </span>
              </div>
              <div className="w-full bg-zinc-900 rounded-full h-2">
                <div
                  className="bg-green-500 h-2 rounded-full"
                  style={{
                    width: `${(metrics.tokens.total_output / metrics.tokens.total) * 100}%`,
                  }}
                />
              </div>

              <div className="pt-3 border-t border-zinc-700">
                <div className="flex items-center justify-between">
                  <span className="text-zinc-400">Total</span>
                  <span className="text-white font-bold">
                    {metrics.tokens.total.toLocaleString()}
                  </span>
                </div>
              </div>
            </div>
          </div>

          {/* Agent Statistics */}
          <div className="bg-zinc-800/50 backdrop-blur rounded-lg border border-zinc-700 p-6">
            <h3 className="text-lg font-semibold text-white mb-4">Agent Statistics</h3>
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-2">
                  <Activity className="w-4 h-4 text-blue-400" />
                  <span className="text-zinc-400">Running</span>
                </div>
                <span className="text-white font-medium">{metrics.agents.running}</span>
              </div>

              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-2">
                  <CheckCircle className="w-4 h-4 text-green-400" />
                  <span className="text-zinc-400">Completed</span>
                </div>
                <span className="text-white font-medium">{metrics.agents.completed}</span>
              </div>

              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-2">
                  <XCircle className="w-4 h-4 text-red-400" />
                  <span className="text-zinc-400">Failed</span>
                </div>
                <span className="text-white font-medium">{metrics.agents.failed}</span>
              </div>

              <div className="pt-3 border-t border-zinc-700">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-2">
                    <Clock className="w-4 h-4 text-zinc-400" />
                    <span className="text-zinc-400">Avg Response Time</span>
                  </div>
                  <span className="text-white font-medium">
                    {metrics.performance.avg_response_time_ms}ms
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Cost Breakdown */}
        <div className="grid grid-cols-2 gap-6">
          <div className="bg-zinc-800/50 backdrop-blur rounded-lg border border-zinc-700 p-6">
            <h3 className="text-lg font-semibold text-white mb-4">Cost by Provider</h3>
            <div className="space-y-3">
              {Object.entries(metrics.costs.by_provider).map(([provider, cost]) => (
                <div key={provider} className="flex items-center justify-between">
                  <span className="text-zinc-400 capitalize">{provider}</span>
                  <span className="text-white font-medium">${cost.toFixed(2)}</span>
                </div>
              ))}
            </div>
          </div>

          <div className="bg-zinc-800/50 backdrop-blur rounded-lg border border-zinc-700 p-6">
            <h3 className="text-lg font-semibold text-white mb-4">Cost by Model</h3>
            <div className="space-y-3">
              {Object.entries(metrics.costs.by_model).map(([model, cost]) => (
                <div key={model} className="flex items-center justify-between">
                  <span className="text-zinc-400">{model}</span>
                  <span className="text-white font-medium">${cost.toFixed(2)}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
