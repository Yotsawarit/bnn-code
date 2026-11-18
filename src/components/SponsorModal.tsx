import React, { useState } from 'react';
import { Heart, ExternalLink, Sparkles, Check, DollarSign } from 'lucide-react';

interface SponsorModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const SponsorModal: React.FC<SponsorModalProps> = ({ isOpen, onClose }) => {
  const [selectedTier, setSelectedTier] = useState<number>(3);
  const [showSimulatedCheckout, setShowSimulatedCheckout] = useState(false);
  const [checkoutDone, setCheckoutDone] = useState(false);

  if (!isOpen) return null;

  const handleSimulatePayment = () => {
    setShowSimulatedCheckout(true);
    setTimeout(() => {
      setCheckoutDone(true);
    }, 1200);
  };

  return (
    <div className="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4 font-mono">
      <div className="bg-[#0e121b] border border-slate-700 rounded-2xl max-w-md w-full p-6 space-y-4 shadow-2xl relative">
        <button
          onClick={onClose}
          className="absolute top-4 right-4 text-slate-400 hover:text-white text-sm cursor-pointer"
        >
          ✕
        </button>

        <div className="text-center space-y-1.5">
          <div className="w-12 h-12 rounded-xl bg-gradient-to-tr from-rose-500 to-pink-600 mx-auto flex items-center justify-center shadow-lg shadow-rose-500/20">
            <Heart className="w-6 h-6 text-white fill-white" />
          </div>
          <h2 className="text-lg font-bold text-white">Support BNN Code Development</h2>
          <p className="text-xs text-slate-400">
            Help fund open-source terminal AI with zero cloud dependencies.
          </p>
        </div>

        {/* Goal Progress Bar */}
        <div className="p-3.5 rounded-xl bg-slate-900/90 border border-slate-800 space-y-2">
          <div className="flex justify-between text-xs">
            <span className="text-slate-400">Monthly Community Goal:</span>
            <span className="text-rose-400 font-bold">$186 / $300 (62%)</span>
          </div>
          <div className="w-full h-2 rounded-full bg-slate-800 overflow-hidden">
            <div className="h-full bg-gradient-to-r from-rose-500 to-pink-500 rounded-full w-[62%]"></div>
          </div>
          <div className="text-[11px] text-slate-500 text-center">
            Target: 100 developers × $3/month = Goal achieved! 🎯
          </div>
        </div>

        {!checkoutDone ? (
          <>
            {/* Tiers */}
            <div className="grid grid-cols-3 gap-2">
              {[
                { amount: 3, label: 'Coffee Tier', desc: 'Backer badge' },
                { amount: 10, label: 'Hacker Tier', desc: 'Priority issues' },
                { amount: 25, label: 'Sponsor Tier', desc: 'Direct roadmap voice' },
              ].map((tier) => (
                <button
                  key={tier.amount}
                  onClick={() => setSelectedTier(tier.amount)}
                  className={`p-2.5 rounded-xl border text-center transition-all cursor-pointer ${
                    selectedTier === tier.amount
                      ? 'bg-rose-950/60 border-rose-500 text-white'
                      : 'bg-slate-900 border-slate-800 text-slate-400 hover:border-slate-700'
                  }`}
                >
                  <div className="text-base font-bold text-rose-300">${tier.amount}</div>
                  <div className="text-[11px] font-semibold text-slate-200 mt-0.5">{tier.label}</div>
                  <div className="text-[9px] text-slate-500">{tier.desc}</div>
                </button>
              ))}
            </div>

            {/* Actions */}
            <div className="space-y-2 pt-2">
              <button
                onClick={handleSimulatePayment}
                disabled={showSimulatedCheckout}
                className="w-full py-2.5 rounded-xl bg-gradient-to-r from-rose-600 to-pink-600 hover:from-rose-500 hover:to-pink-500 text-white text-xs font-bold transition-all shadow-lg shadow-rose-600/20 flex items-center justify-center gap-2 cursor-pointer disabled:opacity-50"
              >
                {showSimulatedCheckout ? (
                  <span className="inline-block animate-spin">⚙</span>
                ) : (
                  <>
                    <DollarSign className="w-4 h-4" />
                    <span>Contribute ${selectedTier} via Stripe</span>
                  </>
                )}
              </button>

              <a
                href="https://github.com/sponsors/Yotsawarit"
                target="_blank"
                rel="noreferrer"
                className="w-full py-2 rounded-xl bg-slate-900 hover:bg-slate-800 border border-slate-700 text-slate-300 text-xs font-semibold transition-colors flex items-center justify-center gap-1.5"
              >
                <span>Sponsor on GitHub</span>
                <ExternalLink className="w-3.5 h-3.5" />
              </a>
            </div>
          </>
        ) : (
          <div className="p-4 rounded-xl bg-emerald-950/40 border border-emerald-800 text-center space-y-2 text-xs">
            <div className="w-10 h-10 rounded-full bg-emerald-900/80 mx-auto flex items-center justify-center">
              <Check className="w-5 h-5 text-emerald-400" />
            </div>
            <p className="font-bold text-emerald-300">Thank you for supporting BNN Code!</p>
            <p className="text-slate-400 text-[11px]">
              Your donation helps keep terminal-native AI fast, open, and free for developers worldwide.
            </p>
            <button
              onClick={onClose}
              className="mt-2 px-4 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white font-semibold cursor-pointer"
            >
              Close
            </button>
          </div>
        )}
      </div>
    </div>
  );
};
