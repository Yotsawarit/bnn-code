import React from 'react';
import { 
  Layers, 
  Terminal, 
  Download, 
  ExternalLink, 
  Check, 
  Copy, 
  Code2, 
  Package, 
  Heart, 
  Keyboard, 
  FileText 
} from 'lucide-react';

interface IntegrationsViewProps {
  onOpenSponsor: () => void;
}

export const IntegrationsView: React.FC<IntegrationsViewProps> = ({ onOpenSponsor }) => {
  const [copiedCmd, setCopiedCmd] = React.useState<string | null>(null);

  const copyCommand = (cmd: string) => {
    navigator.clipboard.writeText(cmd);
    setCopiedCmd(cmd);
    setTimeout(() => setCopiedCmd(null), 2000);
  };

  return (
    <div className="flex-1 flex flex-col h-full max-w-7xl mx-auto w-full p-4 overflow-y-auto space-y-4 font-mono">
      {/* Top Banner */}
      <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-3 p-4 rounded-xl bg-gradient-to-r from-emerald-950/40 via-slate-900 to-slate-900 border border-emerald-900/40">
        <div>
          <div className="flex items-center gap-2">
            <Layers className="w-6 h-6 text-emerald-400" />
            <h1 className="text-lg md:text-xl font-bold text-white">Editor Integrations & Packages</h1>
            <span className="text-[10px] px-2 py-0.5 rounded-full bg-emerald-950 text-emerald-300 border border-emerald-800">
              Official Ecosystem
            </span>
          </div>
          <p className="text-xs text-slate-400 mt-1">
            Zero-friction coding assistant integration for VS Code, Neovim, Linux Debian packages, and Cargo.
          </p>
        </div>

        <button
          onClick={onOpenSponsor}
          className="px-3.5 py-1.5 rounded-lg bg-rose-950/80 hover:bg-rose-900/80 border border-rose-700 text-rose-200 text-xs flex items-center gap-1.5 transition-colors cursor-pointer"
        >
          <Heart className="w-3.5 h-3.5 text-rose-400 fill-rose-400" />
          <span>Support Development ($300/mo Goal)</span>
        </button>
      </div>

      {/* Editor Cards Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* VS Code Extension Card */}
        <div className="p-4 rounded-xl bg-[#0d1117] border border-slate-800 space-y-3 flex flex-col justify-between">
          <div>
            <div className="flex items-center justify-between pb-2 border-b border-slate-800">
              <div className="flex items-center gap-2">
                <Code2 className="w-5 h-5 text-blue-400" />
                <span className="text-sm font-bold text-white">VS Code Extension</span>
              </div>
              <span className="text-[10px] px-1.5 py-0.5 rounded bg-blue-950 border border-blue-800 text-blue-300">
                v0.3.0
              </span>
            </div>

            <p className="text-xs text-slate-300 mt-2 leading-relaxed">
              Native Visual Studio Code extension with context menu actions, keyboard shortcuts, and interactive side panel.
            </p>

            <div className="mt-3 space-y-2">
              <div className="text-xs text-slate-400 font-semibold flex items-center gap-1">
                <Keyboard className="w-3.5 h-3.5 text-cyan-400" />
                Default Keybindings:
              </div>
              <div className="grid grid-cols-2 gap-1.5 text-xs text-slate-300">
                <div className="p-1.5 rounded bg-slate-900 border border-slate-800 flex justify-between">
                  <span>Explain:</span>
                  <kbd className="text-cyan-300 font-bold">Ctrl+Shift+E</kbd>
                </div>
                <div className="p-1.5 rounded bg-slate-900 border border-slate-800 flex justify-between">
                  <span>Refactor:</span>
                  <kbd className="text-cyan-300 font-bold">Ctrl+Shift+R</kbd>
                </div>
                <div className="p-1.5 rounded bg-slate-900 border border-slate-800 flex justify-between">
                  <span>Generate Test:</span>
                  <kbd className="text-cyan-300 font-bold">Ctrl+Shift+T</kbd>
                </div>
                <div className="p-1.5 rounded bg-slate-900 border border-slate-800 flex justify-between">
                  <span>Ask AI Query:</span>
                  <kbd className="text-cyan-300 font-bold">Ctrl+Shift+B</kbd>
                </div>
              </div>
            </div>
          </div>

          <div className="pt-3 border-t border-slate-800 space-y-1.5">
            <div className="text-[11px] text-slate-400">Install via shell:</div>
            <div className="flex items-center justify-between p-2 rounded bg-slate-950 border border-slate-800 text-xs text-cyan-300">
              <code>bash scripts/install-vscode.sh</code>
              <button
                onClick={() => copyCommand('bash scripts/install-vscode.sh')}
                className="text-slate-400 hover:text-white"
              >
                {copiedCmd === 'bash scripts/install-vscode.sh' ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
              </button>
            </div>
          </div>
        </div>

        {/* Neovim Plugin Card */}
        <div className="p-4 rounded-xl bg-[#0d1117] border border-slate-800 space-y-3 flex flex-col justify-between">
          <div>
            <div className="flex items-center justify-between pb-2 border-b border-slate-800">
              <div className="flex items-center gap-2">
                <Terminal className="w-5 h-5 text-emerald-400" />
                <span className="text-sm font-bold text-white">Neovim Plugin (Lua)</span>
              </div>
              <span className="text-[10px] px-1.5 py-0.5 rounded bg-emerald-950 border border-emerald-800 text-emerald-300">
                bnn.nvim
              </span>
            </div>

            <p className="text-xs text-slate-300 mt-2 leading-relaxed">
              First-class Lua plugin integrating with Neovim floating windows, telescope pickers, and terminal buffers.
            </p>

            <div className="mt-3 space-y-2">
              <div className="text-xs text-slate-400 font-semibold flex items-center gap-1">
                <Keyboard className="w-3.5 h-3.5 text-emerald-400" />
                Default Keymaps:
              </div>
              <div className="grid grid-cols-1 gap-1.5 text-xs text-slate-300">
                <div className="p-1.5 rounded bg-slate-900 border border-slate-800 flex justify-between">
                  <span>Open BNN Code Terminal:</span>
                  <kbd className="text-emerald-300 font-bold">&lt;Leader&gt;bn</kbd>
                </div>
                <div className="p-1.5 rounded bg-slate-900 border border-slate-800 flex justify-between">
                  <span>Index Current Workspace:</span>
                  <kbd className="text-emerald-300 font-bold">&lt;Leader&gt;bi</kbd>
                </div>
                <div className="p-1.5 rounded bg-slate-900 border border-slate-800 flex justify-between">
                  <span>Search Codebase Chunks:</span>
                  <kbd className="text-emerald-300 font-bold">&lt;Leader&gt;bs</kbd>
                </div>
              </div>
            </div>
          </div>

          <div className="pt-3 border-t border-slate-800 space-y-1.5">
            <div className="text-[11px] text-slate-400">Install via shell:</div>
            <div className="flex items-center justify-between p-2 rounded bg-slate-950 border border-slate-800 text-xs text-emerald-300">
              <code>bash scripts/install-neovim.sh</code>
              <button
                onClick={() => copyCommand('bash scripts/install-neovim.sh')}
                className="text-slate-400 hover:text-white"
              >
                {copiedCmd === 'bash scripts/install-neovim.sh' ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Linux Packages & Cargo Section */}
      <div className="p-4 rounded-xl bg-[#0d1117] border border-slate-800 space-y-3">
        <div className="flex items-center justify-between pb-2 border-b border-slate-800">
          <div className="flex items-center gap-2">
            <Package className="w-5 h-5 text-orange-400" />
            <span className="text-sm font-bold text-white">Direct Installation Methods</span>
          </div>
          <span className="text-xs text-slate-400">Linux, macOS, Windows</span>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
          <div className="p-3 rounded-lg bg-slate-900 border border-slate-800 space-y-1.5">
            <div className="font-semibold text-xs text-white">Cargo (Rust)</div>
            <p className="text-[11px] text-slate-400">Installs latest release from crates.io.</p>
            <div className="flex items-center justify-between p-1.5 rounded bg-slate-950 text-xs text-cyan-300">
              <code>cargo install bnn-code</code>
              <button onClick={() => copyCommand('cargo install bnn-code')} className="text-slate-400 hover:text-white">
                {copiedCmd === 'cargo install bnn-code' ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
              </button>
            </div>
          </div>

          <div className="p-3 rounded-lg bg-slate-900 border border-slate-800 space-y-1.5">
            <div className="font-semibold text-xs text-white">Debian / Deepin (.deb)</div>
            <p className="text-[11px] text-slate-400">Package for Ubuntu, Debian, Deepin OS.</p>
            <div className="flex items-center justify-between p-1.5 rounded bg-slate-950 text-xs text-cyan-300">
              <code>sudo dpkg -i bnn-code_0.1.3_amd64.deb</code>
              <button onClick={() => copyCommand('sudo dpkg -i bnn-code_0.1.3_amd64.deb')} className="text-slate-400 hover:text-white">
                {copiedCmd === 'sudo dpkg -i bnn-code_0.1.3_amd64.deb' ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
              </button>
            </div>
          </div>

          <div className="p-3 rounded-lg bg-slate-900 border border-slate-800 space-y-1.5">
            <div className="font-semibold text-xs text-white">Pretrained Model Download</div>
            <p className="text-[11px] text-slate-400">Fetches CodeBERTa-Small 1-bit ONNX weights.</p>
            <div className="flex items-center justify-between p-1.5 rounded bg-slate-950 text-xs text-cyan-300">
              <code>bash scripts/download_model.sh --auto</code>
              <button onClick={() => copyCommand('bash scripts/download_model.sh --auto')} className="text-slate-400 hover:text-white">
                {copiedCmd === 'bash scripts/download_model.sh --auto' ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
