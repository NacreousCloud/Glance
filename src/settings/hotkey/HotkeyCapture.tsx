import { useEffect, useState } from 'react';
import type { Corner, HotkeyTrigger } from '../../types';

type Props = {
  value: HotkeyTrigger;
  onChange: (v: HotkeyTrigger) => void;
};

type Mode = 'keyboard' | 'mouse' | 'force_touch' | 'hot_corner';

const MOD_CMD = 1, MOD_CTRL = 2, MOD_SHIFT = 4, MOD_ALT = 8;

const MOUSE_BUTTON_LABEL: Record<number, string> = {
  1: 'Left',
  2: 'Right',
  3: 'Middle',
  4: 'Back',
  5: 'Forward',
};

const CORNER_LABEL: Record<Corner, string> = {
  top_left: 'Top Left',
  top_right: 'Top Right',
  bottom_left: 'Bottom Left',
  bottom_right: 'Bottom Right',
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
    // Initialize a sane default for the new trigger kind so the binding
    // is immediately valid.
    if (next === 'force_touch' && value.kind !== 'force_touch') {
      onChange({ kind: 'force_touch' });
    } else if (next === 'hot_corner' && value.kind !== 'hot_corner') {
      onChange({ kind: 'hot_corner', corner: 'bottom_right', radius_px: 5 });
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
      case 'hot_corner':
        return `Hot Corner: ${CORNER_LABEL[value.corner]} (${value.radius_px}px)`;
    }
  })();

  return (
    <div className="space-y-2">
      <div className="flex flex-wrap gap-2 items-center">
        {tabBtn('keyboard', 'Keyboard')}
        {tabBtn('mouse', 'Mouse')}
        {tabBtn('force_touch', 'Force Touch')}
        {tabBtn('hot_corner', 'Hot Corner')}
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

      {mode === 'hot_corner' && value.kind === 'hot_corner' && (
        <div className="space-y-1">
          <div className="flex items-center gap-2 text-xs">
            <span className="text-gray-500 w-16">Corner</span>
            <select
              className="border rounded px-2 py-1 text-sm flex-1"
              value={value.corner}
              onChange={(e) =>
                onChange({
                  ...value,
                  corner: e.target.value as Corner,
                })
              }
            >
              <option value="top_left">Top Left</option>
              <option value="top_right">Top Right</option>
              <option value="bottom_left">Bottom Left</option>
              <option value="bottom_right">Bottom Right</option>
            </select>
          </div>
          <div className="flex items-center gap-2 text-xs">
            <span className="text-gray-500 w-16">Radius (px)</span>
            <input
              type="number"
              min={1}
              max={100}
              value={value.radius_px}
              onChange={(e) =>
                onChange({
                  ...value,
                  radius_px: Math.max(1, Number(e.target.value) || 1),
                })
              }
              className="border rounded px-2 py-1 w-20"
            />
            <span className="text-gray-400">
              Cursor must enter within this distance of the corner.
            </span>
          </div>
          <p className="text-xs text-gray-400">
            Fires once on cursor entry; the cursor must leave the zone
            before it can re-trigger. Cross-platform.
          </p>
        </div>
      )}
    </div>
  );
}
