import React, { useState } from 'react';
import { 
  Settings, 
  Cpu, 
  Sliders, 
  Save, 
  RotateCcw, 
  Check, 
  Layers, 
  HardDrive, 
  Zap, 
  FileJson
} from 'lucide-react';
import { AVAILABLE_MODELS } from '../services/inferenceEngine';
import { ModelInfo } from '../types';

interface ConfigViewProps {
  activeModel: ModelInfo;
  onModelChange: (model: ModelInfo) => void;
}

export const ConfigView: React.FC<ConfigViewProps> = ({
  activeModel,
  onModelChange,
}) => {
  const [temperature, setTemperature] = useState(activeModel.temperature);
  const [topK, setTopK] = useState(activeModel.topK);
  const [maxTokens, setMaxTokens] = useState(activeModel.maxTokens);
  const [maxFileSizeKb, setMaxFileSizeKb] = useState(100);
  const [streamOutput, setStreamOutput] = useState(true);
  const [showContext, setShowContext] = useState(true);
  const [saved, setSaved] = useState(false);

  const handleSelectModel = (model: ModelInfo) => {
    onModelChange(model);
    setTemperature(model.temperature);
    setTopK(model.topK);
    setMaxTokens(model.maxTokens);
  };

  const handleSave = () => {
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  const jsonConfig = {
    version: '0.1.0',
    model: {
      path: `models/${activeModel.id}/model.onnx`,
      name: activeModel.name,
      quantization: activeModel.quantization,
      max_tokens: maxTokens,
      temperature,
      top_k: topK,
    },
    indexing: {
      max_file_size_kb: maxFileSizeKb,
      exclude_dirs: ['node_modules', 'target', '.git', 'venv', '__pycache__'],
      include_extensions: ['rs', 'py', 'js', 'ts', 'tsx', 'jsx', 'go', 'java', 'cpp', 'c', 'h', 'hpp', 'rb', 'swift', 'kt']
    },
    ui: {
      theme: 'dark',
      show_context: showContext,
      stream_output: streamOutput
    }
  };

  return (
    <div className="flex-1 flex flex-col h-full max-w-7xl mx-auto w-full p-4 overflow-y-auto space-y-4 font-mono">
      {/* Top Banner */}
      <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-3 p-4 rounded-xl bg-gradient-to-r from-slate-900 via-slate-900 to-cyan-950/40 border border-slate-800">
        <div>
          <div className="flex items-center gap-2">
            <Settings className="w-6 h-6 text-cyan-400" />
            <h1 className="text-lg md:text-xl font-bold text-white">BNN Model & Agent Configuration</h1>
            <span className="text-[10px] px-2 py-0.5 rounded-full bg-cyan-950 text-cyan-300 border border-cyan-800">
              .bnn/config.json
            </span>
          </div>
          <p className="text-xs text-slate-400 mt-1">
            Manage quantized BNN neural architectures, SIMD execution modes, and codebase indexing parameters.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={handleSave}
            className="px-4 py-1.5 rounded-lg bg-cyan-600 hover:bg-cyan-500 text-white text-xs font-semibold flex items-center gap-1.5 transition-colors shadow-md cursor-pointer"
          >
            {saved ? <Check className="w-3.5 h-3.5 text-white" /> : <Save className="w-3.5 h-3.5" />}
            <span>{saved ? 'Configuration Saved' : 'Save Config'}</span>
          </button>
        </div>
      </div>

      {/* Model Selection Cards */}
      <div className="space-y-2">
        <h2 className="text-xs font-semibold text-slate-300 flex items-center gap-1.5">
          <Cpu className="w-3.5 h-3.5 text-cyan-400" />
          Available Binarized Neural Networks
        </h2>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
          {AVAILABLE_MODELS.map((model) => {
            const isSelected = activeModel.id === model.id;
            return (
              <div
                key={model.id}
                onClick={() => handleSelectModel(model)}
                className={`p-4 rounded-xl border transition-all cursor-pointer flex flex-col justify-between ${
                  isSelected
                    ? 'bg-cyan-950/40 border-cyan-500 shadow-md shadow-cyan-500/10'
                    : 'bg-[#0d1117] border-slate-800 hover:border-slate-700'
                }`}
              >
                <div>
                  <div className="flex items-center justify-between mb-1.5">
                    <span className="text-sm font-bold text-white">{model.name}</span>
                    {isSelected && (
                      <span className="text-[10px] px-1.5 py-0.5 rounded bg-cyan-900 text-cyan-200 border border-cyan-700 font-semibold">
                        ACTIVE
                      </span>
                    )}
                  </div>
                  <p className="text-xs text-slate-400 leading-relaxed mb-3">
                    {model.description}
                  </p>
                </div>

                <div className="pt-3 border-t border-slate-800/80 space-y-1.5 text-xs">
                  <div className="flex justify-between text-slate-400">
                    <span>Quantization:</span>
                    <span className="text-cyan-300 font-semibold">{model.quantization}</span>
                  </div>
                  <div className="flex justify-between text-slate-400">
                    <span>Memory Footprint:</span>
                    <span className="text-slate-200">{model.sizeMb} MB</span>
                  </div>
                  <div className="flex justify-between text-slate-400">
                    <span>Max Tokens:</span>
                    <span className="text-slate-200">{model.maxTokens}</span>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Settings Grid */}
      <div className="grid grid-cols-1 md:grid-cols-12 gap-3">
        {/* Sliders & Toggles (7 cols) */}
        <div className="md:col-span-7 p-4 rounded-xl bg-[#0d1117] border border-slate-800 space-y-4">
          <div className="flex items-center justify-between pb-2 border-b border-slate-800">
            <span className="text-xs font-semibold text-slate-300 flex items-center gap-1.5">
              <Sliders className="w-3.5 h-3.5 text-purple-400" />
              Inference Hyperparameters
            </span>
            <span className="text-[10px] text-slate-500">Fine-tune generation</span>
          </div>

          {/* Temperature */}
          <div className="space-y-1">
            <div className="flex items-center justify-between text-xs">
              <label className="text-slate-300">Temperature (Creativity):</label>
              <span className="font-bold text-cyan-400">{temperature.toFixed(2)}</span>
            </div>
            <input
              type="range"
              min="0.1"
              max="1.5"
              step="0.05"
              value={temperature}
              onChange={(e) => setTemperature(parseFloat(e.target.value))}
              className="w-full accent-cyan-500 bg-slate-800 rounded-lg cursor-pointer"
            />
            <p className="text-[10px] text-slate-500">Lower values produce deterministic, focused code refactorings; higher values generate diverse test variations.</p>
          </div>

          {/* Top-K */}
          <div className="space-y-1">
            <div className="flex items-center justify-between text-xs">
              <label className="text-slate-300">Top-K Filtering:</label>
              <span className="font-bold text-cyan-400">{topK}</span>
            </div>
            <input
              type="range"
              min="5"
              max="100"
              step="5"
              value={topK}
              onChange={(e) => setTopK(parseInt(e.target.value, 10))}
              className="w-full accent-cyan-500 bg-slate-800 rounded-lg cursor-pointer"
            />
            <p className="text-[10px] text-slate-500">Limits token candidate sampling to the K highest probability predictions.</p>
          </div>

          {/* Max Tokens */}
          <div className="space-y-1">
            <div className="flex items-center justify-between text-xs">
              <label className="text-slate-300">Context & Generation Window:</label>
              <span className="font-bold text-cyan-400">{maxTokens} tokens</span>
            </div>
            <input
              type="range"
              min="512"
              max="8192"
              step="256"
              value={maxTokens}
              onChange={(e) => setMaxTokens(parseInt(e.target.value, 10))}
              className="w-full accent-cyan-500 bg-slate-800 rounded-lg cursor-pointer"
            />
          </div>

          {/* Toggles */}
          <div className="pt-2 border-t border-slate-800/80 space-y-2">
            <label className="flex items-center justify-between cursor-pointer text-xs text-slate-300">
              <span>Enable Real-Time Streaming Output</span>
              <input
                type="checkbox"
                checked={streamOutput}
                onChange={(e) => setStreamOutput(e.target.checked)}
                className="w-4 h-4 accent-cyan-500 rounded"
              />
            </label>

            <label className="flex items-center justify-between cursor-pointer text-xs text-slate-300">
              <span>Show Retrieved Context in Terminal Drawer</span>
              <input
                type="checkbox"
                checked={showContext}
                onChange={(e) => setShowContext(e.target.checked)}
                className="w-4 h-4 accent-cyan-500 rounded"
              />
            </label>
          </div>
        </div>

        {/* Live Config Preview (5 cols) */}
        <div className="md:col-span-5 p-4 rounded-xl bg-[#0d1117] border border-slate-800 flex flex-col">
          <div className="flex items-center justify-between pb-2 border-b border-slate-800 mb-2">
            <span className="text-xs font-semibold text-slate-300 flex items-center gap-1.5">
              <FileJson className="w-3.5 h-3.5 text-yellow-400" />
              .bnn/config.json Preview
            </span>
            <span className="text-[10px] text-emerald-400 font-semibold">Valid JSON</span>
          </div>

          <pre className="flex-1 p-3 rounded-lg bg-slate-950 border border-slate-800/80 text-[11px] text-cyan-300 overflow-x-auto overflow-y-auto leading-relaxed">
            {JSON.stringify(jsonConfig, null, 2)}
          </pre>
        </div>
      </div>
    </div>
  );
};
