import { useState } from 'react';
import type { IconSource } from '../../types';
import { extractAppIcon } from '../api';

type Props = {
  value: IconSource;
  appPath?: string;
  onChange: (icon: IconSource) => void;
};

// Categorized emoji palette for a richer selection experience.
const EMOJI_CATEGORIES: { label: string; emojis: string[] }[] = [
  {
    label: 'Communication',
    emojis: ['💬', '📧', '✉️', '📞', '📱', '🎙️', '📢', '🔔', '💭', '🤝', '👤', '👥']
  },
  {
    label: 'Productivity',
    emojis: ['📝', '📅', '✅', '📁', '📊', '📌', '🔗', '🔍', '💡', '⏰', '⌛', '🚀']
  },
  {
    label: 'System & Tools',
    emojis: ['⚙️', '🛠️', '💻', '🖥️', '🌐', '🏠', '🔧', '🖱️', '🧠', '🛡️', '🔑', '🔋']
  },
  {
    label: 'Media & Design',
    emojis: ['🎵', '🎬', '🎮', '🎨', '📷', '📸', '📚', '🗂️', '🌈', '💎', '🎭', '🎧']
  },
  {
    label: 'Status & Fun',
    emojis: ['⭐', '❤️', '🔥', '🎯', '🏆', '🎉', '✨', '🌟', '🍀', '🍕', '☕', '🥤']
  },
  {
    label: 'Navigation',
    emojis: ['⬅️', '➡️', '⬆️', '⬇️', '🔄', '➕', '➖', '❌', '❓', 'ℹ️', '🚫', '🏁']
  }
];

export default function IconPicker({ value, appPath, onChange }: Props) {
  const [busy, setBusy] = useState(false);

  const tryExtractAppIcon = async () => {
    if (!appPath) return;
    setBusy(true);
    try {
      const base64 = await extractAppIcon(appPath);
      onChange({ kind: 'app_icon_png', base64, source_path: appPath });
    } catch (e) {
      console.error('extract icon failed', e);
    } finally {
      setBusy(false);
    }
  };

  const currentEmoji = value.kind === 'emoji' ? value.value : '';

  return (
    <div className="space-y-6 text-left">
      <div className="space-y-4">
        {EMOJI_CATEGORIES.map((cat) => (
          <div key={cat.label} className="space-y-1.5">
            <h3 className="px-1 text-[11px] font-bold text-ios-label-secondary dark:text-ios-label-secondaryDark uppercase tracking-widest">
              {cat.label}
            </h3>
            <div className="grid grid-cols-6 sm:grid-cols-8 gap-2 bg-black/[0.03] dark:bg-white/5 p-2 rounded-ios">
              {cat.emojis.map((e) => {
                const selected = currentEmoji === e;
                return (
                  <button
                    key={e}
                    type="button"
                    onClick={() => onChange({ kind: 'emoji', value: e })}
                    className={`text-2xl aspect-square flex items-center justify-center rounded-lg transition-all active:scale-90 ${
                      selected 
                        ? 'bg-ios-system-blue text-white shadow-md scale-110 z-10' 
                        : 'hover:bg-black/5 dark:hover:bg-white/10 text-ios-label-primary dark:text-white'
                    }`}
                  >
                    {e}
                  </button>
                );
              })}
            </div>
          </div>
        ))}
      </div>

      <div className="pt-4 border-t border-ios-separator-light dark:border-ios-separator-dark space-y-4">
        <div className="flex items-center justify-between bg-black/[0.03] dark:bg-white/5 p-3 rounded-ios">
          <span className="ios-title">Custom Emoji</span>
          <input
            type="text"
            maxLength={4}
            className="w-16 bg-white dark:bg-white/10 border-none rounded-md px-2 py-1 text-center font-mono text-[17px] focus:ring-2 focus:ring-ios-system-blue outline-none"
            value={currentEmoji}
            placeholder="??"
            onChange={(e) => onChange({ kind: 'emoji', value: e.target.value })}
          />
        </div>

        {appPath && (
          <div className="space-y-2">
            <button
              type="button"
              className={`w-full py-2.5 rounded-ios font-semibold text-[15px] transition-all flex items-center justify-center gap-2 ${
                value.kind === 'app_icon_png'
                  ? 'bg-ios-system-green text-white'
                  : 'bg-ios-system-blue text-white active:opacity-80'
              }`}
              onClick={tryExtractAppIcon}
              disabled={busy}
            >
              {busy ? (
                <>
                  <span className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                  Extracting...
                </>
              ) : value.kind === 'app_icon_png' ? (
                '✓ App Icon Applied'
              ) : (
                'Use App Icon'
              )}
            </button>
            {value.kind === 'app_icon_png' && (
              <p className="px-2 text-[11px] text-ios-label-secondary dark:text-ios-label-secondaryDark truncate italic">
                Source: {value.source_path}
              </p>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
