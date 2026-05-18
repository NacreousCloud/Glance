import { useEffect, useState } from 'react';
import type { HotkeyBinding, HotkeyTrigger } from '../../types';
import {
  listHotkeyBindings,
  upsertHotkeyBinding,
  deleteHotkeyBinding,
} from '../api';
import HotkeyCapture from './HotkeyCapture';

const MOUSE_BUTTON_LABEL: Record<number, string> = {
  1: 'Left', 2: 'Right', 3: 'Middle', 4: 'Back', 5: 'Forward',
};

function formatTrigger(t: HotkeyTrigger): string {
  switch (t.kind) {
    case 'keyboard':
      return t.accelerator || '(none)';
    case 'mouse': {
      const parts: string[] = [];
      if (t.modifiers & 1) parts.push('Cmd');
      if (t.modifiers & 2) parts.push('Ctrl');
      if (t.modifiers & 4) parts.push('Shift');
      if (t.modifiers & 8) parts.push('Alt');
      parts.push(MOUSE_BUTTON_LABEL[t.button] ?? `Mouse${t.button}`);
      return parts.join('+');
    }
    case 'force_touch':
      return 'Force Touch';
    case 'trackpad_tap':
      return `${t.fingers}-finger tap (≤${t.max_duration_ms}ms)`;
  }
}

const newBinding = (): HotkeyBinding => ({
  id: crypto.randomUUID(),
  trigger: { kind: 'keyboard', accelerator: '' },
  menu_mode: 'all',
});

export default function HotkeyEditor() {
  const [bindings, setBindings] = useState<HotkeyBinding[]>([]);
  const [editing, setEditing] = useState<HotkeyBinding | null>(null);

  const refresh = () => listHotkeyBindings().then(setBindings);
  useEffect(() => {
    refresh();
  }, []);

  const onSave = async (b: HotkeyBinding) => {
    await upsertHotkeyBinding(b);
    setEditing(null);
    refresh();
  };

  const onDelete = async (id: string) => {
    await deleteHotkeyBinding(id);
    refresh();
  };

  return (
    <div className="space-y-4">
      {editing ? (
        <div className="ios-card bg-ios-system-blue/[0.03] border-ios-system-blue/20">
          <div className="p-4 space-y-4">
            <h3 className="ios-title text-ios-system-blue">Edit Binding</h3>
            <HotkeyCapture
              value={editing.trigger}
              onChange={(t) => setEditing({ ...editing, trigger: t })}
            />
            <div className="space-y-1">
              <label className="text-[13px] font-semibold text-ios-label-secondary dark:text-ios-label-secondaryDark uppercase tracking-wider px-1">
                Menu Mode
              </label>
              <select
                className="w-full bg-white dark:bg-white/10 border-none rounded-ios px-3 py-2 text-[15px] focus:ring-2 focus:ring-ios-system-blue outline-none"
                value={editing.menu_mode}
                onChange={(e) => setEditing({ ...editing, menu_mode: e.target.value as any })}
              >
                <option value="all">All items</option>
                <option value="launcher">Launcher only</option>
                <option value="notification">Notification actions only</option>
              </select>
            </div>
            <div className="flex gap-2">
              <button
                type="button"
                className="flex-1 py-2.5 bg-ios-system-blue text-white rounded-ios font-semibold active:opacity-80 transition-opacity"
                onClick={() => onSave(editing)}
              >
                Save
              </button>
              <button
                type="button"
                className="flex-1 py-2.5 bg-gray-200 dark:bg-white/10 text-ios-label-primary dark:text-white rounded-ios font-medium active:opacity-80 transition-opacity"
                onClick={() => setEditing(null)}
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      ) : (
        <button
          type="button"
          className="ios-item ios-item-active w-full text-left bg-white/50 dark:bg-white/5 rounded-ios border border-dashed border-ios-separator-light dark:border-ios-separator-dark"
          onClick={() => setEditing(newBinding())}
        >
          <span className="ios-title text-ios-system-blue">+ Add new hotkey</span>
        </button>
      )}

      <div className="ios-card divide-y divide-ios-separator-light dark:divide-ios-separator-dark">
        {bindings.length === 0 && !editing ? (
          <div className="p-8 text-[15px] text-ios-label-secondary dark:text-ios-label-secondaryDark text-center italic">
            No hotkeys configured.
          </div>
        ) : (
          bindings.map((b) => (
            <div key={b.id} className="ios-item group">
              <div className="flex flex-col">
                <span className="ios-title font-mono bg-ios-system-blue/10 dark:bg-ios-system-blue/20 text-ios-system-blue px-2 py-0.5 rounded-md inline-block w-fit">
                  {formatTrigger(b.trigger)}
                </span>
                <span className="ios-subtitle mt-0.5">Mode: {b.menu_mode}</span>
              </div>
              <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                <button 
                  type="button" 
                  className="p-2 text-ios-system-blue font-medium text-[14px] active:opacity-60" 
                  onClick={() => setEditing(b)}
                >
                  Edit
                </button>
                <button 
                  type="button" 
                  className="p-2 text-ios-system-red font-medium text-[14px] active:opacity-60" 
                  onClick={() => onDelete(b.id)}
                >
                  Delete
                </button>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
