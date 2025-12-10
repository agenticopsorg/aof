import { useEffect, useRef, useState } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { Loader2, Pause, Play, StopCircle, Copy, Check } from 'lucide-react';

interface StreamingOutputProps {
  agentId: string;
  onComplete?: () => void;
}

interface TokenMetrics {
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
  tokensPerSecond: number;
  estimatedCost: number;
}

interface StreamChunk {
  content: string;
  timestamp: number;
}

export function StreamingOutput({ agentId, onComplete }: StreamingOutputProps) {
  const [chunks, setChunks] = useState<StreamChunk[]>([]);
  const [isStreaming, setIsStreaming] = useState(true);
  const [isPaused, setIsPaused] = useState(false);
  const [metrics, setMetrics] = useState<TokenMetrics>({
    inputTokens: 0,
    outputTokens: 0,
    totalTokens: 0,
    tokensPerSecond: 0,
    estimatedCost: 0,
  });
  const [copied, setCopied] = useState(false);
  const outputRef = useRef<HTMLDivElement>(null);
  const startTimeRef = useRef<number>(Date.now());

  useEffect(() => {
    let unlistenStream: Promise<UnlistenFn> | null = null;
    let unlistenComplete: Promise<UnlistenFn> | null = null;
    let unlistenMetrics: Promise<UnlistenFn> | null = null;

    // Listen for streaming chunks
    unlistenStream = listen<{ agent_id: string; content: string }>('agent-stream', (event) => {
      if (event.payload.agent_id === agentId && !isPaused) {
        setChunks(prev => [...prev, {
          content: event.payload.content,
          timestamp: Date.now(),
        }]);

        // Auto-scroll to bottom
        setTimeout(() => {
          if (outputRef.current) {
            outputRef.current.scrollTop = outputRef.current.scrollHeight;
          }
        }, 0);

        // Update output tokens
        setMetrics(prev => {
          const newOutputTokens = prev.outputTokens + estimateTokens(event.payload.content);
          const totalTokens = prev.inputTokens + newOutputTokens;
          const elapsedSeconds = (Date.now() - startTimeRef.current) / 1000;
          const tokensPerSecond = elapsedSeconds > 0 ? newOutputTokens / elapsedSeconds : 0;

          return {
            ...prev,
            outputTokens: newOutputTokens,
            totalTokens,
            tokensPerSecond,
            estimatedCost: calculateCost(prev.inputTokens, newOutputTokens),
          };
        });
      }
    });

    // Listen for completion
    unlistenComplete = listen<{ agent_id: string }>('agent-completed', (event) => {
      if (event.payload.agent_id === agentId) {
        setIsStreaming(false);
        onComplete?.();
      }
    });

    // Listen for token metrics updates
    unlistenMetrics = listen<{ agent_id: string; input_tokens: number; output_tokens: number }>(
      'agent-metrics',
      (event) => {
        if (event.payload.agent_id === agentId) {
          setMetrics(prev => ({
            ...prev,
            inputTokens: event.payload.input_tokens,
            outputTokens: event.payload.output_tokens,
            totalTokens: event.payload.input_tokens + event.payload.output_tokens,
            estimatedCost: calculateCost(event.payload.input_tokens, event.payload.output_tokens),
          }));
        }
      }
    );

    return () => {
      unlistenStream?.then(fn => fn());
      unlistenComplete?.then(fn => fn());
      unlistenMetrics?.then(fn => fn());
    };
  }, [agentId, isPaused, onComplete]);

  const fullContent = chunks.map(c => c.content).join('');

  const handleCopy = async () => {
    await navigator.clipboard.writeText(fullContent);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="flex flex-col h-full">
      {/* Metrics Bar */}
      <div className="flex items-center justify-between px-4 py-3 bg-zinc-800/50 border-b border-zinc-700">
        <div className="flex items-center space-x-6 text-sm">
          <div className="flex items-center space-x-2">
            <span className="text-zinc-400">Tokens:</span>
            <span className="text-white font-mono">{metrics.totalTokens.toLocaleString()}</span>
            <span className="text-zinc-500">
              ({metrics.inputTokens} in / {metrics.outputTokens} out)
            </span>
          </div>

          {isStreaming && (
            <div className="flex items-center space-x-2">
              <span className="text-zinc-400">Speed:</span>
              <span className="text-sky-400 font-mono">
                {metrics.tokensPerSecond.toFixed(1)} tok/s
              </span>
            </div>
          )}

          <div className="flex items-center space-x-2">
            <span className="text-zinc-400">Cost:</span>
            <span className="text-green-400 font-mono">
              ${metrics.estimatedCost.toFixed(4)}
            </span>
          </div>
        </div>

        <div className="flex items-center space-x-2">
          {isStreaming && (
            <button
              onClick={() => setIsPaused(!isPaused)}
              className="p-2 rounded-lg bg-zinc-700 hover:bg-zinc-600 transition-colors"
              title={isPaused ? 'Resume' : 'Pause'}
            >
              {isPaused ? (
                <Play className="w-4 h-4 text-white" />
              ) : (
                <Pause className="w-4 h-4 text-white" />
              )}
            </button>
          )}

          <button
            onClick={handleCopy}
            className="p-2 rounded-lg bg-zinc-700 hover:bg-zinc-600 transition-colors"
            title="Copy to clipboard"
          >
            {copied ? (
              <Check className="w-4 h-4 text-green-400" />
            ) : (
              <Copy className="w-4 h-4 text-white" />
            )}
          </button>
        </div>
      </div>

      {/* Streaming Output */}
      <div
        ref={outputRef}
        className="flex-1 overflow-y-auto p-4 font-mono text-sm leading-relaxed"
      >
        {chunks.length === 0 ? (
          <div className="flex items-center justify-center h-full text-zinc-500">
            Waiting for response...
          </div>
        ) : (
          <div className="text-zinc-200 whitespace-pre-wrap">
            {fullContent}
            {isStreaming && !isPaused && (
              <span className="inline-block w-2 h-4 ml-1 bg-sky-400 animate-pulse" />
            )}
          </div>
        )}
      </div>

      {/* Status Bar */}
      {isStreaming && (
        <div className="flex items-center space-x-2 px-4 py-2 bg-zinc-800/50 border-t border-zinc-700">
          <Loader2 className="w-4 h-4 text-sky-400 animate-spin" />
          <span className="text-sm text-zinc-400">
            {isPaused ? 'Paused' : 'Streaming response...'}
          </span>
        </div>
      )}
    </div>
  );
}

// Utility functions
function estimateTokens(text: string): number {
  // Rough estimate: ~4 characters per token
  return Math.ceil(text.length / 4);
}

function calculateCost(inputTokens: number, outputTokens: number): number {
  // Default to Claude Sonnet pricing
  // $3 per million input tokens, $15 per million output tokens
  const inputCost = (inputTokens / 1_000_000) * 3;
  const outputCost = (outputTokens / 1_000_000) * 15;
  return inputCost + outputCost;
}
