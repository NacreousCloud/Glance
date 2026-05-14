import { useEffect, useState } from 'react';
import type { HotkeyTrigger } from '../../types';

type Props = {
  value: HotkeyTrigger;
  onChange: (v: HotkeyTrigger) => void;
};

const MOD_CMD = 1, MOD_CTRL = 2, MOD_SHIFT = 4, MOD_ALT = 8;

export default function HotkeyCapture({ value, onChange }: Props) {
  const [capturing, setCapturing] = useState(false);
  const [mode, setMode] = useState<'keyboard' | 'mouse'>(
    value.kind === 'mouse' ? 'mouse' : 'keyboard'
  );

  useEffect(() => {
    if (!capturing) return;
    const onKey = (e: KeyboardEvent) => {
      if (mode !== 'keyboard') return;
      if (['Meta', 'Control', 'Shift', 'Alt'].includes(e.key)) return;
      e.preventDefault();
      const parts: string[] = [];
      if (e.metaKey) parts.push('CommandOrControl');
      if (e.ctrlKey && !e.metaKey) parts.push('CommandOrControl');
      if (e.shiftKey) parts.push('Shift');
      if (e.altKey) parts.push('Alt');
      parts.push(e.key.toUpperCase());
      onChange({ kind: 'keyboard', accelerator: parts.join('+') });
      setCapturing(false);
    };
    const onMouseDown = (e: MouseEvent) => {
      if (mode !== 'mouse') return;
      // Browser MouseEvent.button: 0=left, 1=middle, 2=right, 3=back, 4=forward
      // Translate to rdev convention: 1=left, 2=right, 3=middle, 4=back, 5=forward
      const map = [1, 3, 2, 4, 5];
      const button = map[e.button] ?? 0;
      if (button === 0) return;
      e.preventDefault();
      let modifiers = 0;
      if (e.metaKey) modifiers |= MOD_CMD;
      if (e.ctrlKey) modifiers |= MOD_CTRL;
      if (e.shiftKey) modifiers |= MOD_SHIFT;
      if (e.altKey) modifiers |= MOD_ALT;
      onChange({ kind: 'mouse', button, modifiers });
      setCapturing(false);
    };
    window.addEventListener('keydown', onKey);
    window.addEventListener('mousedown', onMouseDown);
    return () => {
      window.removeEventListener('keydown', onKey);
      window.removeEventListener('mousedown', onMouseDown);
    };
  }, [capturing, mode, onChange]);

  return (
    <div className="space-y-2">
      <div className="flex gap-2 items-center">
        <button
          type="button"
          className={`px-2 py-1 rounded ${mode === 'keyboard' ? 'bg-blue-100' : 'bg-gray-100'}`}
          onClick={() => setMode('keyboard')}
        >
          Keyboard
        </button>
        <button
          type="button"
          className="px-2 py-1 rounded bg-gray-50 text-gray-400 cursor-not-allowed"
          disabled
          title="Mouse hotkeys are temporarily disabled on macOS (rdev crash workaround). Coming back in a follow-up release."
        >
          Mouse (disabled)
        </button>
      </div>
      <div className="border rounded p-2 flex items-center justify-between">
        <code>
          {value.kind === 'keyboard'
            ? value.accelerator || '(none)'
            : `Mouse ${value.button} + mods=${value.modifiers}`}
        </code>
        <button
          type="button"
          className="px-2 py-1 bg-blue-600 text-white rounded text-sm"
          onClick={() => setCapturing(true)}
        >
          {capturing ? 'Press combo…' : 'Capture'}
        </button>
      </div>
    </div>
  );
}
