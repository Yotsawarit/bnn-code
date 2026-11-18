import React from 'react';
import { 
  Terminal, 
  ShieldAlert, 
  FolderTree, 
  Binary, 
  Settings, 
  Cpu, 
  Heart, 
  Flame,
  Layers
} from 'lucide-react';
import { ModelInfo } from '../types';

interface HeaderProps {
  activeTab: 'terminal' | 'rogue' | 'codebase' | 'pi' | 'config' | 'integrations';
  setActiveTab: (tab: 'terminal' | 'rogue' | 'codebase' | 'pi' | 'config' | 'integrations') => void;
  activeModel: ModelInfo;
  onOpenSponsor: () => void;
  theme: 'dark' | 'matrix' | 'cyberpunk' | 'slate';
  setTheme: (theme: 'dark' | 'matrix' | 'cyberpunk' | 'slate') => void;
}

export const Header: React.FC<HeaderProps> = ({
  activeTab,
  setActiveTab,
  activeModel,
  onOpenSponsor,
  theme,
  setTheme,
}) => {
  return (
    <header className="border-b border-slate-800 bg-[#0d1117]/95 backdrop-blur-md sticky top-0 z-40 px-4 py-2.5">
      <div className="max-w-7xl mx-auto flex flex-col md:flex-row items-center justify-between gap-3">
        {/* Brand & Model Status */}
        <div className="flex items-center gap-3 w-full md:w-auto justify-between md:justify-start">
          <div className="flex items-center gap-2.5 cursor-pointer" onClick={() => setActiveTab('terminal')}>
            <div className="w-9 h-9 rounded-lg bg-gradient-to-br from-cyan-500 via-blue-600 to-indigo-700 p-0.5 shadow-lg shadow-cyan-500/20 flex items-center justify-center">
              <span className="text-xl">🧠</span>
            </div>
            <div>
              <div className="flex items-center gap-2">
                <span className="font-bold font-mono tracking-tight text-white text-base">BNN CODE</span>
                <span className="text-[10px] font-mono font-semibold uppercase px-1.5 py-0.5 rounded bg-cyan-950 border border-cyan-800/80 text-cyan-300">
                  v0.1.3
                </span>
                <span className="hidden sm:inline-block text-[10px] font-mono px-1.5 py-0.5 rounded bg-emerald-950/80 border border-emerald-800/60 text-emerald-400">
                  1-BIT BNN
                </span>
              </div>
              <p className="text-[11px] text-slate-400 font-mono hidden sm:block">
                Terminal-native AI coding agent powered by BNNs
              </p>
            </div>
          </div>

          <div className="flex md:hidden items-center gap-2">
            <button
              onClick={onOpenSponsor}
              className="px-2 py-1 rounded bg-rose-950/80 border border-rose-800 text-rose-300 text-xs flex items-center gap-1 font-mono hover:bg-rose-900/80 transition-colors"
            >
              <Heart className="w-3 h-3 text-rose-400 fill-rose-400" />
              Sponsor
            </button>
          </div>
        </div>

        {/* Navigation Tabs */}
        <nav className="flex items-center gap-1 overflow-x-auto max-w-full pb-1 md:pb-0 scrollbar-none">
          <button
            onClick={() => setActiveTab('terminal')}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-mono font-medium transition-all whitespace-nowrap ${
              activeTab === 'terminal'
                ? 'bg-cyan-500/15 text-cyan-300 border border-cyan-500/40 shadow-sm shadow-cyan-500/10'
                : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/60 border border-transparent'
            }`}
          >
            <Terminal className="w-3.5 h-3.5 text-cyan-400" />
            <span>Terminal REPL</span>
          </button>

          <button
            onClick={() => setActiveTab('rogue')}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-mono font-medium transition-all whitespace-nowrap ${
              activeTab === 'rogue'
                ? 'bg-amber-500/15 text-amber-300 border border-amber-500/40 shadow-sm shadow-amber-500/10'
                : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/60 border border-transparent'
            }`}
          >
            <ShieldAlert className="w-3.5 h-3.5 text-amber-400" />
            <span>Rogue Suite</span>
          </button>

          <button
            onClick={() => setActiveTab('codebase')}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-mono font-medium transition-all whitespace-nowrap ${
              activeTab === 'codebase'
                ? 'bg-blue-500/15 text-blue-300 border border-blue-500/40 shadow-sm shadow-blue-500/10'
                : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/60 border border-transparent'
            }`}
          >
            <FolderTree className="w-3.5 h-3.5 text-blue-400" />
            <span>AST Indexer</span>
          </button>

          <button
            onClick={() => setActiveTab('pi')}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-mono font-medium transition-all whitespace-nowrap ${
              activeTab === 'pi'
                ? 'bg-purple-500/15 text-purple-300 border border-purple-500/40 shadow-sm shadow-purple-500/10'
                : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/60 border border-transparent'
            }`}
          >
            <Binary className="w-3.5 h-3.5 text-purple-400" />
            <span>Pi Math Lab</span>
          </button>

          <button
            onClick={() => setActiveTab('config')}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-mono font-medium transition-all whitespace-nowrap ${
              activeTab === 'config'
                ? 'bg-slate-700/60 text-slate-200 border border-slate-600'
                : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/60 border border-transparent'
            }`}
          >
            <Settings className="w-3.5 h-3.5 text-slate-400" />
            <span>Config & Model</span>
          </button>

          <button
            onClick={() => setActiveTab('integrations')}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-mono font-medium transition-all whitespace-nowrap ${
              activeTab === 'integrations'
                ? 'bg-emerald-500/15 text-emerald-300 border border-emerald-500/40'
                : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/60 border border-transparent'
            }`}
          >
            <Layers className="w-3.5 h-3.5 text-emerald-400" />
            <span>Integrations</span>
          </button>
        </nav>

        {/* Right Actions: Model Pill, Theme Selector, Sponsor */}
        <div className="hidden md:flex items-center gap-2.5">
          <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-slate-900 border border-slate-800 text-xs font-mono text-slate-300">
            <Cpu className="w-3.5 h-3.5 text-cyan-400 animate-pulse" />
            <span className="text-slate-400">Model:</span>
            <span className="text-cyan-300 font-semibold">{activeModel.name.split(' ')[0]}</span>
          </div>

          {/* Theme switcher */}
          <select
            value={theme}
            onChange={(e) => setTheme(e.target.value as any)}
            className="bg-slate-900 border border-slate-800 text-slate-300 text-xs font-mono rounded-md px-2 py-1 focus:outline-none focus:border-cyan-500 cursor-pointer"
            title="Terminal Theme"
          >
            <option value="dark">Dark Theme</option>
            <option value="matrix">Matrix Green</option>
            <option value="cyberpunk">Cyberpunk Neon</option>
            <option value="slate">Slate Minimal</option>
          </select>

          <button
            onClick={onOpenSponsor}
            className="px-3 py-1 rounded-md bg-gradient-to-r from-rose-950 to-pink-950 border border-rose-800/80 text-rose-200 text-xs flex items-center gap-1.5 font-mono font-medium hover:border-rose-600 transition-all shadow-sm"
          >
            <Heart className="w-3.5 h-3.5 text-rose-400 fill-rose-400" />
            <span>Support</span>
          </button>
        </div>
      </div>
    </header>
  );
};
