import { useState } from 'react';
import type { IconSource } from '../../types';
import { extractAppIcon } from '../api';

type Props = {
  value: IconSource;
  appPath?: string;
  onChange: (icon: IconSource) => void;
};

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

  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2">
        <label className="text-sm font-medium">Icon (emoji)</label>
        <input
          type="text"
          maxLength={4}
          className="border rounded px-2 py-1 w-16 text-center"
          value={value.kind === 'emoji' ? value.value : ''}
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
