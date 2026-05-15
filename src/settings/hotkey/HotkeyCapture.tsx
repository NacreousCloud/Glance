import { useEffect, useState } from 'react';
import type { HotkeyTrigger } from '../../types';

type Props = {
  value: HotkeyTrigger;
  onChange: (v: HotkeyTrigger) => void;
};

type Mode = 'keyboard' | 'mouse' | 'force_touch' | 'trackpad_tap';

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

function modeOf(value: HotkeyTrigger): Mode {
  return value.kind as Mode;
}

export default function HotkeyCapture({ value, onChange }: Props) {
  const [capturing, setCapturing] = useState(false);
  const [mode, setMode] = useState<Mode>(modeOf(value));

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
      const map = [1, 3, 2, 4, 5];
      const button = map[e.button] ?? 0;
      if (button === 0) return;
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

  const switchMode = (next: Mode) => {
    setMode(next);
    setCapturing(false);
    if (next === 'force_touch' && value.kind !== 'force_touch') {
      onChange({ kind: 'force_touch' });
    } else if (next === 'trackpad_tap' && value.kind !== 'trackpad_tap') {
      onChange({ kind: 'trackpad_tap', fingers: 3, max_duration_ms: 200 });
    }
  };

  const tabBtn = (m: Mode, label: string) => (
    <button
      type="button"
      className={`px-2 py-1 rounded ${
        mode === m ? 'bg-blue-100' : 'bg-gray-100'
      }`}
      onClick={() => switchMode(m)}
    >
      {label}
    </button>
  );

  const display = (() => {
    switch (value.kind) {
      case 'keyboard':
        return value.accelerator || '(none)';
      case 'mouse':
        return formatMouseBinding(value.button, value.modifiers);
      case 'force_touch':
        return 'Force Touch (macOS only)';
      case 'trackpad_tap':
        return `${value.fingers}-finger tap (≤${value.max_duration_ms}ms)`;
    }
  })();

  return (
    <div className="space-y-2">
      <div className="flex flex-wrap gap-2 items-center">
        {tabBtn('keyboard', 'Keyboard')}
        {tabBtn('mouse', 'Mouse')}
        {tabBtn('force_touch', 'Force Touch')}
        {tabBtn('trackpad_tap', 'Trackpad Tap')}
      </div>

      <div className="border rounded p-2 flex items-center justify-between gap-2">
        <code className="truncate">{display}</code>
        {(mode === 'keyboard' || mode === 'mouse') && (
          <button
            type="button"
            className="px-2 py-1 bg-blue-600 text-white rounded text-sm shrink-0"
            onClick={() => setCapturing(true)}
          >
            {capturing ? 'Press combo…' : 'Capture'}
          </button>
        )}
      </div>

      {mode === 'mouse' && (
        <p className="text-xs text-gray-500">
          Capture a mouse button 3+ (middle / back / forward). Left/right
          are intentionally blocked. Requires Accessibility permission.
          Combine with modifiers (Cmd/Shift/Alt/Ctrl) to avoid colliding
          with normal clicks.
        </p>
      )}

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
            browser; type them here. Valid modifiers:{' '}
            <code>CommandOrControl</code>, <code>Shift</code>,{' '}
            <code>Alt</code>, <code>Super</code>.
          </p>
        </div>
      )}

      {mode === 'force_touch' && (
        <p className="text-xs text-gray-500">
          Force-click on a Force Touch trackpad fires the menu. macOS
          only. Disable the system "Force Click and haptic feedback"
          look-up behavior if it conflicts, or stack this with another
          trigger.
        </p>
      )}

      {mode === 'trackpad_tap' && value.kind === 'trackpad_tap' && (
        <div className="space-y-1">
          <div className="flex items-center gap-2 text-xs">
            <span className="text-gray-500 w-24">Fingers</span>
            <input
              type="number"
              min={2}
              max={5}
              value={value.fingers}
              onChange={(e) =>
                onChange({
                  ...value,
                  fingers: Math.min(5, Math.max(2, Number(e.target.value) || 3)),
                })
              }
              className="border rounded px-2 py-1 w-16"
            />
          </div>
          <div className="flex items-center gap-2 text-xs">
            <span className="text-gray-500 w-24">Max duration</span>
            <input
              type="number"
              min={50}
              max={1000}
              step={10}
              value={value.max_duration_ms}
              onChange={(e) =>
                onChange({
                  ...value,
                  max_duration_ms: Math.max(50, Number(e.target.value) || 200),
                })
              }
              className="border rounded px-2 py-1 w-20"
            />
            <span className="text-gray-400">ms</span>
          </div>
          <p className="text-xs text-amber-700 bg-amber-50 border border-amber-200 rounded p-2">
            ⚠️ Uses Apple's private <code>MultitouchSupport.framework</code>.
            Works on built-in trackpads and Magic Trackpad. May break on
            future macOS releases. Not App-Store-distributable.
          </p>
        </div>
      )}
    </div>
  );
}
