import { useEffect, useState } from 'react';
import type { HotkeyBinding } from '../../types';
import {
  listHotkeyBindings,
  upsertHotkeyBinding,
  deleteHotkeyBinding,
} from '../api';
import HotkeyCapture from './HotkeyCapture';

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
    <section className="space-y-3">
      <h2 className="text-sm font-semibold">Hotkey bindings</h2>
      {editing ? (
        <div className="border p-3 rounded space-y-2">
          <HotkeyCapture
            value={editing.trigger}
            onChange={(t) => setEditing({ ...editing, trigger: t })}
          />
          <div>
            <label className="text-sm font-medium">Menu mode</label>
            <select
              className="border rounded px-2 py-1 ml-2"
              value={editing.menu_mode}
              onChange={(e) => setEditing({ ...editing, menu_mode: e.target.value })}
            >
              <option value="all">All items</option>
              <option value="launcher">Launcher only</option>
              <option value="notification">Notification actions only</option>
            </select>
          </div>
          <div className="flex gap-2">
            <button
              type="button"
              className="px-3 py-1 bg-blue-600 text-white rounded"
              onClick={() => onSave(editing)}
            >
              Save
            </button>
            <button
              type="button"
              className="px-3 py-1 bg-gray-200 rounded"
              onClick={() => setEditing(null)}
            >
              Cancel
            </button>
          </div>
        </div>
      ) : (
        <button
          type="button"
          className="px-3 py-1 bg-gray-200 rounded text-sm"
          onClick={() => setEditing(newBinding())}
        >
          + Add binding
        </button>
      )}
      <ul className="space-y-1">
        {bindings.map((b) => (
          <li
            key={b.id}
            className="flex items-center justify-between border rounded p-2"
          >
            <code className="text-sm">
              {b.trigger.kind === 'keyboard'
                ? b.trigger.accelerator
                : `Mouse ${b.trigger.button} + mods=${b.trigger.modifiers}`}
              {' → '}
              {b.menu_mode}
            </code>
            <div className="flex gap-1">
              <button type="button" className="px-2 text-sm" onClick={() => setEditing(b)}>Edit</button>
              <button type="button" className="px-2 text-sm text-red-600" onClick={() => onDelete(b.id)}>Delete</button>
            </div>
          </li>
        ))}
      </ul>
    </section>
  );
}
