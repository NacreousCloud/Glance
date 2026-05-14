import { useEffect, useState } from 'react';
import type { HotkeyTrigger } from '../../types';

type Props = {
  value: HotkeyTrigger;
  onChange: (v: HotkeyTrigger) => void;
};

const MOD_CMD = 1, MOD_CTRL = 2, MOD_SHIFT = 4, MOD_ALT = 8;

const MOUSE_BUTTON_LABEL: Record<number, string> = {
  1: 'Left',
  2: 'Right',
  3: 'Middle',
  4: 'Back',
  5: 'Forward',
};

function formatMouseBinding(button: number, mods: number): string {
  const parts: string[] = [];
  if (mods & MOD_CMD) parts.push('Cmd');
  if (mods & MOD_CTRL) parts.push('Ctrl');
  if (mods & MOD_SHIFT) parts.push('Shift');
  if (mods & MOD_ALT) parts.push('Alt');
  const label = MOUSE_BUTTON_LABEL[button] ?? `Mouse${button}`;
  parts.push(label);
  return parts.join('+');
}

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
      // Translate to settings convention: 1=left, 2=right, 3=middle, 4=back, 5=forward
      const map = [1, 3, 2, 4, 5];
      const button = map[e.button] ?? 0;
      if (button === 0) return;
      // Backend only observes OtherMouseDown (buttons 3+). Left/right
      // capture is rejected so users do not save a binding that never
      // fires.
      if (button === 1 || button === 2) {
        alert(
          'Left and right mouse buttons cannot be used as hotkeys. ' +
            'Pick middle / back / forward.'
        );
        return;
      }
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
          className={`px-2 py-1 rounded ${mode === 'mouse' ? 'bg-blue-100' : 'bg-gray-100'}`}
          onClick={() => setMode('mouse')}
        >
          Mouse
        </button>
      </div>
      {mode === 'mouse' && (
        <p className="text-xs text-gray-500">
          Capture a mouse button 3+ (middle / back / forward). Left/right
          are intentionally blocked. Requires Accessibility permission.
          Combine with modifiers (Cmd/Shift/Alt/Ctrl) to avoid colliding
          with normal clicks.
        </p>
      )}
      <div className="border rounded p-2 flex items-center justify-between">
        <code>
          {value.kind === 'keyboard'
            ? value.accelerator || '(none)'
            : formatMouseBinding(value.button, value.modifiers)}
        </code>
        <button
          type="button"
          className="px-2 py-1 bg-blue-600 text-white rounded text-sm"
          onClick={() => setCapturing(true)}
        >
          {capturing ? 'Press combo…' : 'Capture'}
        </button>
      </div>
      {mode === 'keyboard' && (
        <div className="space-y-1">
          <label className="text-xs text-gray-500">
            Or type the accelerator manually (e.g. <code>F24</code>,{' '}
            <code>CommandOrControl+Shift+M</code>):
          </label>
          <input
            type="text"
            className="border rounded px-2 py-1 w-full font-mono text-sm"
            placeholder="F24"
            value={value.kind === 'keyboard' ? value.accelerator : ''}
            onChange={(e) =>
              onChange({ kind: 'keyboard', accelerator: e.target.value })
            }
          />
          <p className="text-xs text-gray-400">
            F13–F24 and other special keys often aren't captured by the
            browser; type them here. Valid modifiers: <code>CommandOrControl</code>,{' '}
            <code>Shift</code>, <code>Alt</code>, <code>Super</code>.
          </p>
        </div>
      )}
    </div>
  );
}
