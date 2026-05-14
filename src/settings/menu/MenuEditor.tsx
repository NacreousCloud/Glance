import { useEffect, useState } from 'react';
import {
  listMenuItems,
  upsertMenuItem,
  deleteMenuItem,
  reorderMenuItems,
} from '../api';
import type { MenuItem } from '../../types';
import MenuItemForm from './MenuItemForm';

const newItem = (): MenuItem => ({
  id: crypto.randomUUID(),
  label: 'New item',
  icon: { kind: 'emoji', value: '⚡' },
  action: { kind: 'launch_app', path: '' },
  tags: ['launcher'],
});

export default function MenuEditor() {
  const [items, setItems] = useState<MenuItem[]>([]);
  const [editing, setEditing] = useState<MenuItem | null>(null);

  const refresh = () => listMenuItems().then(setItems);
  useEffect(() => {
    refresh();
  }, []);

  const onSave = async (item: MenuItem) => {
    await upsertMenuItem(item);
    setEditing(null);
    refresh();
  };

  const onDelete = async (id: string) => {
    await deleteMenuItem(id);
    refresh();
  };

  const moveUp = async (i: number) => {
    if (i === 0) return;
    const next = items.slice();
    [next[i - 1], next[i]] = [next[i], next[i - 1]];
    await reorderMenuItems(next.map((n) => n.id));
    refresh();
  };

  const moveDown = async (i: number) => {
    if (i === items.length - 1) return;
    const next = items.slice();
    [next[i + 1], next[i]] = [next[i], next[i + 1]];
    await reorderMenuItems(next.map((n) => n.id));
    refresh();
  };

  return (
    <section className="space-y-3">
      <h2 className="text-sm font-semibold">Menu items</h2>
      {editing ? (
        <MenuItemForm
          initial={editing}
          onSave={onSave}
          onCancel={() => setEditing(null)}
        />
      ) : (
        <button
          type="button"
          className="px-3 py-1 bg-gray-200 rounded text-sm"
          onClick={() => setEditing(newItem())}
        >
          + Add item
        </button>
      )}
      <ul className="space-y-1">
        {items.map((it, i) => (
          <li
            key={it.id}
            className="flex items-center justify-between border rounded p-2"
          >
            <div className="flex items-center gap-2">
              <span className="text-xl">
                {it.icon.kind === 'emoji' ? (
                  it.icon.value
                ) : (
                  <img src={`data:image/png;base64,${it.icon.base64}`} width={24} height={24} alt="" />
                )}
              </span>
              <span>{it.label}</span>
            </div>
            <div className="flex gap-1">
              <button type="button" className="px-2 text-sm" onClick={() => moveUp(i)}>↑</button>
              <button type="button" className="px-2 text-sm" onClick={() => moveDown(i)}>↓</button>
              <button type="button" className="px-2 text-sm" onClick={() => setEditing(it)}>Edit</button>
              <button type="button" className="px-2 text-sm text-red-600" onClick={() => onDelete(it.id)}>Delete</button>
            </div>
          </li>
        ))}
      </ul>
    </section>
  );
}
