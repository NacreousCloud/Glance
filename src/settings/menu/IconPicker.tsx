import { useState } from 'react';
import type { IconSource } from '../../types';
import { extractAppIcon } from '../api';

type Props = {
  value: IconSource;
  appPath?: string;
  onChange: (icon: IconSource) => void;
};

// Curated 40-emoji palette for radial menu items. Covers common app /
// action categories so the user rarely needs to type emoji manually.
const EMOJI_PRESETS: string[] = [
  '💬', '📧', '📱', '📝', '📅', '✅', '📁', '📊',
  '⚙️', '🛠️', '💻', '🖥️', '🌐', '🔍', '🏠', '📌',
  '🎵', '🎬', '🎮', '🎨', '📷', '📚', '🗂️', '🔗',
  '⭐', '❤️', '💡', '🚀', '⚡', '🔥', '🎯', '🏆',
  '🎉', '✨', '💎', '🌟', '⏰', '🔧', '🖱️', '🧠',
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
    <div className="space-y-2">
      <div>
        <label className="text-sm font-medium">Icon</label>
        <div className="mt-2 grid grid-cols-8 gap-1 border rounded p-2">
          {EMOJI_PRESETS.map((e) => {
            const selected = currentEmoji === e;
            return (
              <button
                key={e}
                type="button"
                onClick={() => onChange({ kind: 'emoji', value: e })}
                className={`text-2xl leading-none aspect-square rounded hover:bg-gray-100 ${
                  selected ? 'bg-blue-100 ring-2 ring-blue-400' : ''
                }`}
                title={e}
              >
                {e}
              </button>
            );
          })}
        </div>
      </div>

      <div className="flex items-center gap-2">
        <label className="text-xs text-gray-500">Or custom emoji:</label>
        <input
          type="text"
          maxLength={4}
          className="border rounded px-2 py-1 w-16 text-center"
          value={currentEmoji}
          onChange={(e) => onChange({ kind: 'emoji', value: e.target.value })}
        />
      </div>

      {appPath && (
        <button
          type="button"
          className="px-2 py-1 bg-gray-100 rounded text-sm"
          onClick={tryExtractAppIcon}
          disabled={busy}
        >
          {busy ? 'Extracting…' : 'Use app icon'}
        </button>
      )}
      {value.kind === 'app_icon_png' && (
        <div className="text-xs text-gray-500">
          App icon from <code>{value.source_path}</code>
        </div>
      )}
    </div>
  );
}
