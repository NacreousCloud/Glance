import { useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import type { MenuItem, Action } from '../../types';
import IconPicker from './IconPicker';
import { SHELL_PRESETS, presetsForCurrentPlatform } from './shellPresets';

type Props = {
  initial: MenuItem;
  onSave: (item: MenuItem) => void;
  onCancel: () => void;
};

export default function MenuItemForm({ initial, onSave, onCancel }: Props) {
  const [item, setItem] = useState<MenuItem>(initial);
  const [shellAdvanced, setShellAdvanced] = useState(false);
  const appPath =
    item.action.kind === 'launch_app' ? item.action.path : undefined;

  const updateAction = (a: Action) => setItem({ ...item, action: a });

  const applyShellPreset = (presetId: string) => {
    const p = SHELL_PRESETS.find((x) => x.id === presetId);
    if (!p) return;
    setItem((prev) => ({
      ...prev,
      // Auto-fill label + icon from preset if they look default.
      label:
        prev.label.trim() && prev.label.trim() !== 'New item'
          ? prev.label
          : p.label,
      icon:
        prev.icon.kind === 'emoji' && (prev.icon.value === '⚡' || prev.icon.value === '')
          ? { kind: 'emoji', value: p.icon }
          : prev.icon,
      action: {
        kind: 'run_shell',
        command: p.command,
        args: p.args,
        confirm: p.confirm,
      },
    }));
  };

  return (
    <div className="space-y-6 text-left">
      <div className="space-y-1.5">
        <label className="px-1 text-[13px] uppercase tracking-wider text-ios-label-secondary dark:text-ios-label-secondaryDark font-semibold">
          General
        </label>
        <div className="ios-card divide-y divide-ios-separator-light dark:divide-ios-separator-dark bg-white/50 dark:bg-white/5">
          <div className="ios-item flex-col items-stretch py-3">
            <span className="ios-subtitle mb-1 px-1">Label</span>
            <input
              type="text"
              className="w-full bg-transparent border-none p-1 text-[17px] focus:ring-0 outline-none"
              value={item.label}
              placeholder="Menu item name"
              onChange={(e) => setItem({ ...item, label: e.target.value })}
            />
          </div>
          <div className="ios-item">
            <span className="ios-title">Action Type</span>
            <select
              className="bg-transparent border-none text-ios-system-blue font-medium text-[15px] outline-none cursor-pointer text-right appearance-none"
              value={item.action.kind}
              onChange={(e) => {
                const kind = e.target.value as Action['kind'];
                if (kind === 'launch_app') updateAction({ kind: 'launch_app', path: '' });
                else if (kind === 'open_url') updateAction({ kind: 'open_url', url: '' });
                else updateAction({ kind: 'run_shell', command: '', args: [], confirm: true });
              }}
            >
              <option value="launch_app">Launch App</option>
              <option value="open_url">Open URL</option>
              <option value="run_shell">Shell Command</option>
            </select>
          </div>
        </div>
      </div>

      <div className="space-y-1.5">
        <label className="px-1 text-[13px] uppercase tracking-wider text-ios-label-secondary dark:text-ios-label-secondaryDark font-semibold">
          Action Configuration
        </label>
        <div className="ios-card bg-white/50 dark:bg-white/5 p-4">
          {item.action.kind === 'launch_app' && (
            <div className="flex gap-2">
              <input
                type="text"
                placeholder="/Applications/Slack.app"
                className="bg-gray-100 dark:bg-white/10 border-none rounded-ios px-3 py-2 flex-1 text-[14px] outline-none"
                value={item.action.path}
                onChange={(e) => updateAction({ kind: 'launch_app', path: e.target.value })}
              />
              <button
                type="button"
                className="px-4 py-2 bg-ios-system-blue text-white rounded-ios text-[14px] font-semibold active:opacity-80 transition-opacity"
                onClick={async () => {
                  const picked = await open({
                    multiple: false,
                    directory: false,
                    filters: [{ name: 'Applications', extensions: ['app'] }],
                    defaultPath: '/Applications',
                  });
                  if (typeof picked === 'string') {
                    const itemLabel = item.label.trim();
                    const fallbackLabel = picked.split('/').pop()?.replace(/\.app$/i, '') ?? '';
                    setItem((prev) => ({
                      ...prev,
                      action: { kind: 'launch_app', path: picked },
                      label: itemLabel && itemLabel !== 'New item' ? itemLabel : fallbackLabel,
                    }));
                  }
                }}
              >
                Browse
              </button>
            </div>
          )}
          {item.action.kind === 'open_url' && (
            <input
              type="text"
              placeholder="https://example.com"
              className="w-full bg-gray-100 dark:bg-white/10 border-none rounded-ios px-3 py-2 text-[14px] outline-none"
              value={item.action.url}
              onChange={(e) => updateAction({ kind: 'open_url', url: e.target.value })}
            />
          )}
          {item.action.kind === 'run_shell' && (() => {
            const shell = item.action;
            return (
              <div className="space-y-4">
                <div className="grid grid-cols-2 gap-1.5 max-h-48 overflow-y-auto pr-1">
                  {presetsForCurrentPlatform().map((p) => {
                    const selected = shell.command === p.command && shell.args.length === p.args.length && shell.args.every((a, i) => a === p.args[i]);
                    return (
                      <button
                        key={p.id}
                        type="button"
                        onClick={() => applyShellPreset(p.id)}
                        className={`flex items-center gap-2 px-3 py-2 rounded-ios text-[13px] text-left transition-all ${
                          selected ? 'bg-ios-system-blue text-white' : 'bg-gray-100 dark:bg-white/10 hover:bg-gray-200'
                        }`}
                      >
                        <span className="text-base">{p.icon}</span>
                        <span className="truncate font-medium">{p.label}</span>
                      </button>
                    );
                  })}
                </div>

                <div className="pt-2 space-y-3 border-t border-ios-separator-light dark:border-ios-separator-dark">
                  <label className="flex items-center justify-between">
                    <span className="ios-subtitle">Manual Edit</span>
                    <input
                      type="checkbox"
                      className="w-10 h-5 appearance-none bg-gray-300 dark:bg-white/10 rounded-full relative transition-colors checked:bg-ios-system-blue before:content-[''] before:absolute before:w-4 before:h-4 before:bg-white before:rounded-full before:top-0.5 before:left-0.5 before:transition-transform checked:before:translate-x-5"
                      checked={shellAdvanced}
                      onChange={(e) => setShellAdvanced(e.target.checked)}
                    />
                  </label>

                  {shellAdvanced && (
                    <div className="space-y-2">
                      <input
                        type="text"
                        placeholder="Command (full path)"
                        className="w-full bg-gray-100 dark:bg-white/10 border-none rounded-ios px-3 py-2 font-mono text-[12px] outline-none"
                        value={shell.command}
                        onChange={(e) => updateAction({ ...shell, command: e.target.value })}
                      />
                      <input
                        type="text"
                        placeholder="Arguments"
                        className="w-full bg-gray-100 dark:bg-white/10 border-none rounded-ios px-3 py-2 font-mono text-[12px] outline-none"
                        value={shell.args.join(' ')}
                        onChange={(e) => updateAction({ ...shell, args: e.target.value.split(/\s+/).filter((a) => a.length > 0) })}
                      />
                    </div>
                  )}

                  <label className="flex items-center justify-between">
                    <span className="ios-subtitle">Confirmation Prompt</span>
                    <input
                      type="checkbox"
                      className="w-10 h-5 appearance-none bg-gray-300 dark:bg-white/10 rounded-full relative transition-colors checked:bg-ios-system-green before:content-[''] before:absolute before:w-4 before:h-4 before:bg-white before:rounded-full before:top-0.5 before:left-0.5 before:transition-transform checked:before:translate-x-5"
                      checked={shell.confirm}
                      onChange={(e) => updateAction({ ...shell, confirm: e.target.checked })}
                    />
                  </label>
                </div>
              </div>
            );
          })()}
        </div>
      </div>

      <div className="space-y-1.5">
        <label className="px-1 text-[13px] uppercase tracking-wider text-ios-label-secondary dark:text-ios-label-secondaryDark font-semibold">
          Appearance
        </label>
        <div className="ios-card bg-white/50 dark:bg-white/5 p-4">
          <IconPicker
            value={item.icon}
            appPath={appPath}
            onChange={(icon) => setItem({ ...item, icon })}
          />
        </div>
      </div>

      <div className="space-y-1.5">
        <label className="px-1 text-[13px] uppercase tracking-wider text-ios-label-secondary dark:text-ios-label-secondaryDark font-semibold">
          Visibility
        </label>
        <div className="ios-card divide-y divide-ios-separator-light dark:divide-ios-separator-dark bg-white/50 dark:bg-white/5">
          <label className="ios-item">
            <span className="ios-title">Launcher Menu</span>
            <input
              type="checkbox"
              className="w-11 h-6 appearance-none bg-gray-300 dark:bg-white/10 rounded-full relative transition-colors checked:bg-ios-system-green before:content-[''] before:absolute before:w-5 before:h-5 before:bg-white before:rounded-full before:top-0.5 before:left-0.5 before:transition-transform checked:before:translate-x-5"
              checked={item.tags.includes('launcher')}
              onChange={(e) => setItem({ ...item, tags: e.target.checked ? Array.from(new Set([...item.tags, 'launcher'])) : item.tags.filter((t) => t !== 'launcher') })}
            />
          </label>
          <label className="ios-item">
            <span className="ios-title">Notification Context</span>
            <input
              type="checkbox"
              className="w-11 h-6 appearance-none bg-gray-300 dark:bg-white/10 rounded-full relative transition-colors checked:bg-ios-system-green before:content-[''] before:absolute before:w-5 before:h-5 before:bg-white before:rounded-full before:top-0.5 before:left-0.5 before:transition-transform checked:before:translate-x-5"
              checked={item.tags.includes('notification')}
              onChange={(e) => setItem({ ...item, tags: e.target.checked ? Array.from(new Set([...item.tags, 'notification'])) : item.tags.filter((t) => t !== 'notification') })}
            />
          </label>
        </div>
      </div>

      <div className="flex gap-3 pt-2">
        <button
          type="button"
          className="flex-1 py-3 bg-ios-system-blue text-white rounded-ios font-bold text-[17px] active:opacity-80 transition-opacity shadow-lg shadow-blue-500/20"
          onClick={() => onSave(item)}
        >
          Save Item
        </button>
        <button
          type="button"
          className="px-6 py-3 bg-gray-200 dark:bg-white/10 text-ios-label-primary dark:text-white rounded-ios font-semibold text-[17px] active:opacity-80 transition-opacity"
          onClick={onCancel}
        >
          Cancel
        </button>
      </div>
    </div>
  );
}
