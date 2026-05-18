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
    <div className="space-y-4">
      {editing ? (
        <div className="ios-card bg-ios-system-blue/[0.03] border-ios-system-blue/20">
          <div className="p-4">
            <MenuItemForm
              initial={editing}
              onSave={onSave}
              onCancel={() => setEditing(null)}
            />
          </div>
        </div>
      ) : (
        <button
          type="button"
          className="ios-item ios-item-active w-full text-left bg-white/50 dark:bg-white/5 rounded-ios border border-dashed border-ios-separator-light dark:border-ios-separator-dark"
          onClick={() => setEditing(newItem())}
        >
          <span className="ios-title text-ios-system-blue">+ Add menu item</span>
        </button>
      )}

      <div className="ios-card divide-y divide-ios-separator-light dark:divide-ios-separator-dark">
        {items.length === 0 && !editing ? (
          <div className="p-8 text-[15px] text-ios-label-secondary dark:text-ios-label-secondaryDark text-center italic">
            No items in radial menu.
          </div>
        ) : (
          items.map((it, i) => (
            <div key={it.id} className="ios-item group transition-colors hover:bg-black/[0.01] dark:hover:bg-white/[0.01]">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-ios bg-gray-100 dark:bg-white/10 flex items-center justify-center text-xl shadow-sm">
                  {it.icon.kind === 'emoji' ? (
                    it.icon.value
                  ) : (
                    <img src={`data:image/png;base64,${it.icon.base64}`} className="w-6 h-6 object-contain" alt="" />
                  )}
                </div>
                <div className="flex flex-col">
                  <span className="ios-title">{it.label}</span>
                  <span className="ios-subtitle">{it.action.kind.replace('_', ' ')}</span>
                </div>
              </div>

              <div className="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                <div className="flex flex-col">
                  <button type="button" className="p-1.5 text-ios-label-secondary hover:text-ios-system-blue transition-colors disabled:opacity-20" disabled={i === 0} onClick={() => moveUp(i)}>
                    <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M5 15l7-7 7 7" /></svg>
                  </button>
                  <button type="button" className="p-1.5 text-ios-label-secondary hover:text-ios-system-blue transition-colors disabled:opacity-20" disabled={i === items.length - 1} onClick={() => moveDown(i)}>
                    <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M19 9l-7 7-7-7" /></svg>
                  </button>
                </div>
                <div className="w-px h-8 bg-ios-separator-light dark:bg-ios-separator-dark mx-1" />
                <button type="button" className="p-2 text-ios-system-blue font-medium text-[14px] active:opacity-60" onClick={() => setEditing(it)}>
                  Edit
                </button>
                <button type="button" className="p-2 text-ios-system-red font-medium text-[14px] active:opacity-60" onClick={() => onDelete(it.id)}>
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
