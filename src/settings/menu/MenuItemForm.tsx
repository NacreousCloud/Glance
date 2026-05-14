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
    <div className="space-y-3 border p-4 rounded">
      <div>
        <label className="text-sm font-medium">Label</label>
        <input
          type="text"
          className="border rounded px-2 py-1 w-full"
          value={item.label}
          onChange={(e) => setItem({ ...item, label: e.target.value })}
        />
      </div>
      <div>
        <label className="text-sm font-medium">Action type</label>
        <select
          className="border rounded px-2 py-1"
          value={item.action.kind}
          onChange={(e) => {
            const kind = e.target.value as Action['kind'];
            if (kind === 'launch_app') updateAction({ kind: 'launch_app', path: '' });
            else if (kind === 'open_url') updateAction({ kind: 'open_url', url: '' });
            else updateAction({ kind: 'run_shell', command: '', args: [], confirm: true });
          }}
        >
          <option value="launch_app">Launch app</option>
          <option value="open_url">Open URL</option>
          <option value="run_shell">Run shell command</option>
        </select>
      </div>
      {item.action.kind === 'launch_app' && (
        <div className="flex gap-2">
          <input
            type="text"
            placeholder="/Applications/Slack.app"
            className="border rounded px-2 py-1 flex-1"
            value={item.action.path}
            onChange={(e) => updateAction({ kind: 'launch_app', path: e.target.value })}
          />
          <button
            type="button"
            className="px-3 py-1 bg-gray-100 rounded text-sm"
            onClick={async () => {
              const picked = await open({
                multiple: false,
                directory: false,
                filters: [{ name: 'Applications', extensions: ['app'] }],
                defaultPath: '/Applications',
              });
              if (typeof picked === 'string') {
                const itemLabel = item.label.trim();
                const fallbackLabel = picked
                  .split('/')
                  .pop()
                  ?.replace(/\.app$/i, '')
                  ?? '';
                setItem((prev) => ({
                  ...prev,
                  action: { kind: 'launch_app', path: picked },
                  // Auto-fill label from app name if user hasn't set one.
                  label: itemLabel && itemLabel !== 'New item' ? itemLabel : fallbackLabel,
                }));
              }
            }}
          >
            Browse…
          </button>
        </div>
      )}
      {item.action.kind === 'open_url' && (
        <input
          type="text"
          placeholder="https://example.com"
          className="border rounded px-2 py-1 w-full"
          value={item.action.url}
          onChange={(e) => updateAction({ kind: 'open_url', url: e.target.value })}
        />
      )}
      {item.action.kind === 'run_shell' && (() => {
        const shell = item.action;
        return (
          <div className="space-y-3">
            <div>
              <label className="text-sm font-medium">Preset</label>
              <div className="mt-1 grid grid-cols-2 gap-1 border rounded p-2 max-h-64 overflow-y-auto">
                {presetsForCurrentPlatform().map((p) => {
                  const selected =
                    shell.command === p.command &&
                    shell.args.length === p.args.length &&
                    shell.args.every((a, i) => a === p.args[i]);
                  return (
                    <button
                      key={p.id}
                      type="button"
                      onClick={() => applyShellPreset(p.id)}
                      className={`flex items-center gap-2 px-2 py-1 rounded text-sm text-left hover:bg-gray-100 ${
                        selected ? 'bg-blue-100 ring-2 ring-blue-400' : ''
                      }`}
                      title={
                        p.description
                          ? `${p.command} ${p.args.join(' ')}\n\n${p.description}`
                          : `${p.command} ${p.args.join(' ')}`
                      }
                    >
                      <span className="text-lg">{p.icon}</span>
                      <span className="truncate">{p.label}</span>
                    </button>
                  );
                })}
              </div>
              {(() => {
                const selected = SHELL_PRESETS.find(
                  (p) =>
                    shell.command === p.command &&
                    shell.args.length === p.args.length &&
                    shell.args.every((a, i) => a === p.args[i])
                );
                if (!selected?.description) return null;
                return (
                  <p className="mt-2 text-xs text-amber-700 bg-amber-50 border border-amber-200 rounded p-2">
                    ⚠️ {selected.description}
                  </p>
                );
              })()}
            </div>

            <label className="flex items-center gap-2 text-xs text-gray-500">
              <input
                type="checkbox"
                checked={shellAdvanced}
                onChange={(e) => setShellAdvanced(e.target.checked)}
              />
              Advanced — edit command manually
            </label>

            {shellAdvanced && (
              <div className="space-y-2 border-l-2 border-gray-200 pl-3">
                <input
                  type="text"
                  placeholder="/usr/local/bin/foo (full path)"
                  className="border rounded px-2 py-1 w-full font-mono text-sm"
                  value={shell.command}
                  onChange={(e) =>
                    updateAction({ ...shell, command: e.target.value })
                  }
                />
                <input
                  type="text"
                  placeholder="args (space-separated)"
                  className="border rounded px-2 py-1 w-full font-mono text-sm"
                  value={shell.args.join(' ')}
                  onChange={(e) =>
                    updateAction({
                      ...shell,
                      args: e.target.value.split(/\s+/).filter((a) => a.length > 0),
                    })
                  }
                />
                <p className="text-xs text-gray-500">
                  Runs via <code>std::process::Command</code> — not a shell.
                  Use <code>/bin/sh -c "..."</code> as command for pipes,
                  redirects, <code>~</code> expansion, or env vars.
                </p>
              </div>
            )}

            <label className="flex items-center gap-2 text-sm">
              <input
                type="checkbox"
                checked={shell.confirm}
                onChange={(e) =>
                  updateAction({ ...shell, confirm: e.target.checked })
                }
              />
              Confirm before running
            </label>
          </div>
        );
      })()}
      <IconPicker
        value={item.icon}
        appPath={appPath}
        onChange={(icon) => setItem({ ...item, icon })}
      />
      <div>
        <label className="text-sm font-medium">Show in modes</label>
        <div className="flex gap-3 mt-1">
          <label className="flex items-center gap-1 text-sm">
            <input
              type="checkbox"
              checked={item.tags.includes('launcher')}
              onChange={(e) =>
                setItem({
                  ...item,
                  tags: e.target.checked
                    ? Array.from(new Set([...item.tags, 'launcher']))
                    : item.tags.filter((t) => t !== 'launcher'),
                })
              }
            />
            Launcher
          </label>
          <label className="flex items-center gap-1 text-sm">
            <input
              type="checkbox"
              checked={item.tags.includes('notification')}
              onChange={(e) =>
                setItem({
                  ...item,
                  tags: e.target.checked
                    ? Array.from(new Set([...item.tags, 'notification']))
                    : item.tags.filter((t) => t !== 'notification'),
                })
              }
            />
            Notification
          </label>
        </div>
      </div>
      <div className="flex gap-2">
        <button
          type="button"
          className="px-3 py-1 bg-blue-600 text-white rounded"
          onClick={() => onSave(item)}
        >
          Save
        </button>
        <button
          type="button"
          className="px-3 py-1 bg-gray-200 rounded"
          onClick={onCancel}
        >
          Cancel
        </button>
      </div>
    </div>
  );
}
