import React, { useState, useRef, useEffect } from 'react';
import { 
  Play, 
  Trash2, 
  Copy, 
  Check, 
  Sparkles, 
  Cpu, 
  Layers, 
  HelpCircle, 
  CornerDownLeft, 
  FileCode2,
  Tv
} from 'lucide-react';
import { CodeChunk, ModelInfo, TerminalLog } from '../types';
import { globalInferenceEngine } from '../services/inferenceEngine';

interface TerminalViewProps {
  activeModel: ModelInfo;
  theme: 'dark' | 'matrix' | 'cyberpunk' | 'slate';
  onNavigateTab: (tab: 'terminal' | 'rogue' | 'codebase' | 'pi' | 'config' | 'integrations') => void;
}

export const TerminalView: React.FC<TerminalViewProps> = ({
  activeModel,
  theme,
  onNavigateTab,
}) => {
  const [inputCommand, setInputCommand] = useState('');
  const [logs, setLogs] = useState<TerminalLog[]>([
    {
      id: 'init-1',
      timestamp: new Date().toLocaleTimeString(),
      command: 'bnn --version',
      output: 'bnn-code 0.1.3 (x86_64-unknown-linux-gnu)\nTerminal-native AI coding agent powered by BNNs\nEngine: ONNX Runtime v1.17 + SSE4.2 Vectorized SIMD',
      category: 'system'
    },
    {
      id: 'init-2',
      timestamp: new Date().toLocaleTimeString(),
      command: 'bnn stats',
      output: 'Codebase indexed: 5 files, 18 AST chunks\nModel loaded: CodeBERTa-Small (1-bit BNN) | Context: 4096 tokens\nType "help" for commands, or select a preset below to try BNN Code!',
      category: 'info'
    }
  ]);
  const [history, setHistory] = useState<string[]>([
    'bnn stats',
    'bnn explain src/main.rs',
    'bnn rogue',
    'bnn pi --digits 200 --bench'
  ]);
  const [historyIndex, setHistoryIndex] = useState<number>(-1);
  const [isExecuting, setIsExecuting] = useState(false);
  const [lastRetrievedChunks, setLastRetrievedChunks] = useState<CodeChunk[]>([]);
  const [showScanlines, setShowScanlines] = useState(false);
  const [copiedId, setCopiedId] = useState<string | null>(null);

  const logsEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    logsEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [logs]);

  const handleRunCommand = async (cmdToRun: string) => {
    if (!cmdToRun.trim() || isExecuting) return;

    const trimmed = cmdToRun.trim();
    if (trimmed.toLowerCase() === 'clear') {
      setLogs([]);
      setInputCommand('');
      globalInferenceEngine.clearHistory();
      return;
    }

    setIsExecuting(true);
    setHistory(prev => [trimmed, ...prev.filter(h => h !== trimmed)].slice(0, 50));
    setHistoryIndex(-1);

    const logId = `log-${Date.now()}`;
    const newLog: TerminalLog = {
      id: logId,
      timestamp: new Date().toLocaleTimeString(),
      command: trimmed,
      output: '🧠 Thinking with BNN inference engine...',
      isStreaming: true,
      category: 'command'
    };

    setLogs(prev => [...prev, newLog]);
    setInputCommand('');

    try {
      const result = await globalInferenceEngine.executeCommand(trimmed);
      setLastRetrievedChunks(result.retrievedChunks);

      setLogs(prev =>
        prev.map(l =>
          l.id === logId
            ? {
                ...l,
                output: result.stdout,
                isStreaming: false,
                category: trimmed.startsWith('bnn rogue') ? 'info' : 'response'
              }
            : l
        )
      );
    } catch (err: any) {
      setLogs(prev =>
        prev.map(l =>
          l.id === logId
            ? {
                ...l,
                output: `Error executing command: ${err.message || 'Unknown error'}`,
                isStreaming: false,
                isError: true,
              }
            : l
        )
      );
    } finally {
      setIsExecuting(false);
      setTimeout(() => inputRef.current?.focus(), 100);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleRunCommand(inputCommand);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (history.length > 0 && historyIndex < history.length - 1) {
        const nextIndex = historyIndex + 1;
        setHistoryIndex(nextIndex);
        setInputCommand(history[nextIndex]);
      }
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (historyIndex > 0) {
        const nextIndex = historyIndex - 1;
        setHistoryIndex(nextIndex);
        setInputCommand(history[nextIndex]);
      } else if (historyIndex === 0) {
        setHistoryIndex(-1);
        setInputCommand('');
      }
    }
  };

  const copyLogOutput = (id: string, text: string) => {
    navigator.clipboard.writeText(text);
    setCopiedId(id);
    setTimeout(() => setCopiedId(null), 2000);
  };

  // Color schemes for terminal
  const themeStyles = {
    dark: {
      bg: 'bg-[#090d16]',
      border: 'border-slate-800',
      text: 'text-slate-100',
      prompt: 'text-cyan-400',
      highlight: 'text-cyan-300',
    },
    matrix: {
      bg: 'bg-[#041008]',
      border: 'border-emerald-900/60',
      text: 'text-emerald-300',
      prompt: 'text-emerald-400 font-bold',
      highlight: 'text-emerald-200',
    },
    cyberpunk: {
      bg: 'bg-[#120824]',
      border: 'border-purple-900/60',
      text: 'text-pink-100',
      prompt: 'text-yellow-400 font-bold',
      highlight: 'text-cyan-300',
    },
    slate: {
      bg: 'bg-[#0f172a]',
      border: 'border-slate-700',
      text: 'text-slate-200',
      prompt: 'text-blue-400',
      highlight: 'text-indigo-300',
    }
  }[theme];

  const quickCommands = [
    { label: 'bnn explain src/main.rs', desc: 'Architecture breakdown' },
    { label: 'bnn rogue', desc: 'Anomaly & threat detection' },
    { label: 'bnn pi --digits 500 --bench', desc: 'Arbitrary-precision math' },
    { label: 'bnn refactor src/main.rs', desc: 'Clean code & speedup' },
    { label: 'bnn test src/math.rs', desc: 'Synthesize unit tests' },
    { label: 'bnn commit', desc: 'Conventional git commit' },
    { label: 'bnn review', desc: 'Multi-category code audit' },
    { label: 'bnn fix', desc: 'Compiler error scan' }
  ];

  return (
    <div className="flex-1 flex flex-col md:flex-row h-full max-w-7xl mx-auto w-full overflow-hidden p-3 gap-3">
      {/* Main Terminal Window (Ratatui style) */}
      <div className={`flex-1 flex flex-col rounded-xl border ${themeStyles.border} ${themeStyles.bg} overflow-hidden shadow-2xl relative`}>
        {showScanlines && <div className="absolute inset-0 scanlines pointer-events-none z-20 opacity-40"></div>}

        {/* Ratatui Title Bar */}
        <div className="bg-slate-900/90 border-b border-slate-800/80 px-3 py-2 flex items-center justify-between z-10">
          <div className="flex items-center gap-2">
            <div className="flex gap-1.5 mr-2">
              <span className="w-3 h-3 rounded-full bg-red-500/80 inline-block"></span>
              <span className="w-3 h-3 rounded-full bg-yellow-500/80 inline-block"></span>
              <span className="w-3 h-3 rounded-full bg-green-500/80 inline-block"></span>
            </div>
            <div className="font-mono text-xs font-semibold text-slate-300 flex items-center gap-2">
              <span className="text-cyan-400">┌─</span>
              <span>🧠 BNN Terminal REPL</span>
              <span className="text-slate-500">──</span>
              <span className="text-slate-400 font-normal hidden sm:inline">Ratatui + Crossterm Console</span>
              <span className="text-cyan-400">─┐</span>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <button
              onClick={() => setShowScanlines(!showScanlines)}
              className={`p-1.5 rounded text-xs font-mono flex items-center gap-1 border transition-all ${
                showScanlines
                  ? 'bg-cyan-950 text-cyan-300 border-cyan-700'
                  : 'text-slate-400 border-slate-800 hover:text-slate-200'
              }`}
              title="Toggle Retro CRT Scanlines"
            >
              <Tv className="w-3.5 h-3.5" />
              <span className="text-[11px] hidden sm:inline">CRT</span>
            </button>

            <button
              onClick={() => setLogs([])}
              className="p-1.5 rounded text-slate-400 hover:text-slate-200 hover:bg-slate-800 text-xs font-mono transition-colors border border-slate-800"
              title="Clear Terminal Output"
            >
              <Trash2 className="w-3.5 h-3.5" />
            </button>
          </div>
        </div>

        {/* Terminal Scroll Area */}
        <div className="flex-1 overflow-y-auto p-4 font-mono text-xs md:text-sm space-y-4 select-text">
          {logs.map((log) => (
            <div key={log.id} className="group relative">
              {/* Command row */}
              <div className="flex items-start gap-2 text-slate-400 mb-1">
                <span className={themeStyles.prompt}>bnn&gt;</span>
                <span className="text-white font-medium break-all">{log.command}</span>
                <span className="text-[10px] text-slate-600 ml-auto whitespace-nowrap">{log.timestamp}</span>
              </div>

              {/* Output row */}
              <div className="relative pl-5 border-l-2 border-slate-800/80">
                <pre className={`whitespace-pre-wrap break-words leading-relaxed font-mono ${
                  log.isError 
                    ? 'text-rose-400 bg-rose-950/20 p-2 rounded border border-rose-900/40' 
                    : log.isStreaming 
                    ? 'text-cyan-400 animate-pulse' 
                    : themeStyles.text
                }`}>
                  {log.output}
                </pre>

                {/* Copy output button on hover */}
                {!log.isStreaming && log.output && (
                  <button
                    onClick={() => copyLogOutput(log.id, log.output)}
                    className="absolute top-0 right-0 p-1 rounded bg-slate-800/80 hover:bg-slate-700 text-slate-300 opacity-0 group-hover:opacity-100 transition-opacity"
                    title="Copy output"
                  >
                    {copiedId === log.id ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
                  </button>
                )}
              </div>
            </div>
          ))}
          <div ref={logsEndRef} />
        </div>

        {/* Ratatui Status Bar */}
        <div className="bg-slate-950/90 border-t border-slate-800/80 px-3 py-1 text-[11px] font-mono text-slate-400 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span className={`inline-block w-2 h-2 rounded-full ${isExecuting ? 'bg-amber-400 animate-ping' : 'bg-emerald-400'}`}></span>
            <span className="text-slate-300 font-semibold">{isExecuting ? 'INFERENCE ACTIVE' : 'READY'}</span>
            <span className="text-slate-600">|</span>
            <span className="hidden sm:inline text-slate-400">Model: {activeModel.name}</span>
          </div>
          <div className="text-slate-500 text-[10px]">
            <span>Press <kbd className="px-1 py-0.5 rounded bg-slate-800 text-slate-300">Enter</kbd> to run</span>
            <span className="mx-1">·</span>
            <span><kbd className="px-1 py-0.5 rounded bg-slate-800 text-slate-300">↑/↓</kbd> history</span>
          </div>
        </div>

        {/* Command Input Bar */}
        <div className="p-3 bg-slate-900/90 border-t border-slate-800 flex items-center gap-2">
          <span className={`font-mono font-bold ${themeStyles.prompt} pl-1`}>bnn&gt;</span>
          <input
            ref={inputRef}
            type="text"
            value={inputCommand}
            onChange={(e) => setInputCommand(e.target.value)}
            onKeyDown={handleKeyDown}
            disabled={isExecuting}
            placeholder={isExecuting ? 'Processing neural inference...' : 'Type command (e.g. explain src/main.rs, pi, rogue, help)...'}
            className="flex-1 bg-transparent font-mono text-xs md:text-sm text-slate-100 placeholder-slate-500 focus:outline-none"
            autoFocus
          />
          <button
            onClick={() => handleRunCommand(inputCommand)}
            disabled={!inputCommand.trim() || isExecuting}
            className="px-3 py-1.5 rounded-lg bg-cyan-600 hover:bg-cyan-500 disabled:opacity-40 disabled:hover:bg-cyan-600 text-white font-mono text-xs flex items-center gap-1.5 transition-colors shadow-sm cursor-pointer"
          >
            {isExecuting ? (
              <span className="inline-block animate-spin">⚙</span>
            ) : (
              <>
                <span>Run</span>
                <CornerDownLeft className="w-3.5 h-3.5" />
              </>
            )}
          </button>
        </div>
      </div>

      {/* Right Sidebar: Context Drawer & Presets */}
      <div className="w-full md:w-80 flex flex-col gap-3">
        {/* Quick Presets Box */}
        <div className="rounded-xl border border-slate-800 bg-[#0d1117] p-3 flex flex-col gap-2">
          <div className="flex items-center justify-between pb-2 border-b border-slate-800/80">
            <div className="flex items-center gap-1.5 text-xs font-mono font-semibold text-slate-200">
              <Sparkles className="w-3.5 h-3.5 text-cyan-400" />
              <span>Quick Commands</span>
            </div>
            <span className="text-[10px] font-mono text-slate-500">1-click execute</span>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-1 gap-1.5 max-h-52 overflow-y-auto">
            {quickCommands.map((q, idx) => (
              <button
                key={idx}
                onClick={() => handleRunCommand(q.label)}
                disabled={isExecuting}
                className="text-left px-2.5 py-1.5 rounded-lg bg-slate-900/80 hover:bg-slate-800/80 border border-slate-800 text-slate-300 hover:text-cyan-300 font-mono text-xs transition-all flex flex-col cursor-pointer"
              >
                <span className="font-semibold text-slate-200">{q.label}</span>
                <span className="text-[10px] text-slate-500">{q.desc}</span>
              </button>
            ))}
          </div>
        </div>

        {/* Retrieved Context Chunks */}
        <div className="flex-1 rounded-xl border border-slate-800 bg-[#0d1117] p-3 flex flex-col gap-2 min-h-48">
          <div className="flex items-center justify-between pb-2 border-b border-slate-800/80">
            <div className="flex items-center gap-1.5 text-xs font-mono font-semibold text-slate-200">
              <Layers className="w-3.5 h-3.5 text-purple-400" />
              <span>Retrieved AST Context</span>
            </div>
            <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-slate-800 text-slate-400">
              {lastRetrievedChunks.length} chunks
            </span>
          </div>

          <div className="flex-1 overflow-y-auto space-y-2 pr-1">
            {lastRetrievedChunks.length === 0 ? (
              <div className="h-full flex flex-col items-center justify-center text-center p-4 text-slate-500 font-mono text-xs">
                <FileCode2 className="w-8 h-8 text-slate-600 mb-2" />
                <p>No active query context.</p>
                <p className="text-[10px] text-slate-600 mt-1">Run an explain, test, or query to inspect AST chunks pulled from the index.</p>
              </div>
            ) : (
              lastRetrievedChunks.map((chunk, idx) => (
                <div key={idx} className="p-2 rounded-lg bg-slate-900 border border-slate-800/80 font-mono text-xs">
                  <div className="flex items-center justify-between text-[11px] text-cyan-400 mb-1">
                    <span className="font-semibold truncate max-w-[170px]">{chunk.filePath}</span>
                    <span className="text-[10px] text-slate-500">L{chunk.startLine}-{chunk.endLine}</span>
                  </div>
                  {chunk.symbolName && (
                    <div className="text-[10px] text-purple-300 mb-1">
                      symbol: <span className="text-white font-bold">{chunk.symbolName}</span> ({chunk.chunkType})
                    </div>
                  )}
                  <pre className="text-[11px] text-slate-300 line-clamp-3 bg-slate-950 p-1.5 rounded border border-slate-800/50 overflow-hidden">
                    {chunk.content}
                  </pre>
                </div>
              ))
            )}
          </div>
        </div>

        {/* Model Spec Card */}
        <div className="rounded-xl border border-slate-800 bg-[#0d1117] p-3 text-xs font-mono">
          <div className="flex items-center justify-between text-slate-300 mb-1.5">
            <div className="flex items-center gap-1.5 text-cyan-400 font-semibold">
              <Cpu className="w-3.5 h-3.5" />
              <span>BNN Runtime Status</span>
            </div>
            <span className="text-[10px] text-emerald-400">SSE4.2 OK</span>
          </div>
          <div className="grid grid-cols-2 gap-2 text-[11px] text-slate-400 pt-1 border-t border-slate-800/60">
            <div>Quantization: <span className="text-slate-200">1-bit Binary</span></div>
            <div>Footprint: <span className="text-slate-200">{activeModel.sizeMb} MB</span></div>
            <div>Max Tokens: <span className="text-slate-200">{activeModel.maxTokens}</span></div>
            <div>Architecture: <span className="text-slate-200">ONNX BNN</span></div>
          </div>
        </div>
      </div>
    </div>
  );
};
