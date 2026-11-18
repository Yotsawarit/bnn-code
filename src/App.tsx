import React, { useState } from 'react';
import { Header } from './components/Header';
import { TerminalView } from './components/TerminalView';
import { RogueView } from './components/RogueView';
import { CodebaseView } from './components/CodebaseView';
import { PiLabView } from './components/PiLabView';
import { ConfigView } from './components/ConfigView';
import { IntegrationsView } from './components/IntegrationsView';
import { SponsorModal } from './components/SponsorModal';
import { AVAILABLE_MODELS } from './services/inferenceEngine';
import { ModelInfo } from './types';

export function App() {
  const [activeTab, setActiveTab] = useState<'terminal' | 'rogue' | 'codebase' | 'pi' | 'config' | 'integrations'>('terminal');
  const [activeModel, setActiveModel] = useState<ModelInfo>(AVAILABLE_MODELS[0]);
  const [theme, setTheme] = useState<'dark' | 'matrix' | 'cyberpunk' | 'slate'>('dark');
  const [isSponsorOpen, setIsSponsorOpen] = useState(false);

  return (
    <div className={`min-h-screen flex flex-col ${
      theme === 'matrix' ? 'bg-[#020b04] text-emerald-200' :
      theme === 'cyberpunk' ? 'bg-[#0b0314] text-pink-100' :
      theme === 'slate' ? 'bg-[#0b1120] text-slate-200' :
      'bg-[#090d16] text-[#e2e8f0]'
    }`}>
      {/* Top Application Bar */}
      <Header
        activeTab={activeTab}
        setActiveTab={setActiveTab}
        activeModel={activeModel}
        onOpenSponsor={() => setIsSponsorOpen(true)}
        theme={theme}
        setTheme={setTheme}
      />

      {/* Main View Area */}
      <main className="flex-1 flex flex-col min-h-0 relative overflow-hidden">
        {activeTab === 'terminal' && (
          <TerminalView
            activeModel={activeModel}
            theme={theme}
            onNavigateTab={setActiveTab}
          />
        )}
        {activeTab === 'rogue' && <RogueView />}
        {activeTab === 'codebase' && <CodebaseView />}
        {activeTab === 'pi' && <PiLabView />}
        {activeTab === 'config' && (
          <ConfigView
            activeModel={activeModel}
            onModelChange={setActiveModel}
          />
        )}
        {activeTab === 'integrations' && (
          <IntegrationsView onOpenSponsor={() => setIsSponsorOpen(true)} />
        )}
      </main>

      {/* Persistent Bottom Status Bar */}
      <footer className="border-t border-slate-800/80 bg-[#070a10] px-4 py-1.5 text-[11px] font-mono text-slate-500 flex flex-col sm:flex-row items-center justify-between gap-2 z-30">
        <div className="flex items-center gap-3">
          <span className="flex items-center gap-1.5 text-slate-400">
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 inline-block animate-pulse"></span>
            BNN Inference Engine: <span className="text-emerald-400 font-semibold">Online</span>
          </span>
          <span className="hidden sm:inline text-slate-700">|</span>
          <span className="hidden sm:inline text-slate-400">SIMD: SSE4.2 Vectorized</span>
          <span className="hidden sm:inline text-slate-700">|</span>
          <span className="hidden md:inline text-slate-400">Memory: ~{activeModel.sizeMb}MB</span>
        </div>

        <div className="flex items-center gap-4 text-slate-400">
          <a
            href="https://crates.io/crates/bnn-code"
            target="_blank"
            rel="noreferrer"
            className="hover:text-cyan-400 transition-colors"
          >
            crates.io/bnn-code
          </a>
          <span className="text-slate-700">·</span>
          <a
            href="https://github.com/Yotsawarit/bnn-code"
            target="_blank"
            rel="noreferrer"
            className="hover:text-cyan-400 transition-colors"
          >
            GitHub: PAC-CHAT/bnn-code
          </a>
          <span className="text-slate-700">·</span>
          <span>MIT License</span>
        </div>
      </footer>

      {/* Community Sponsor Modal */}
      <SponsorModal
        isOpen={isSponsorOpen}
        onClose={() => setIsSponsorOpen(false)}
      />
    </div>
  );
}

export default App;
