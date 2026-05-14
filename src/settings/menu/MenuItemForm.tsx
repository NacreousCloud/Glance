import { useState } from 'react';
import type { MenuItem, Action } from '../../types';
import IconPicker from './IconPicker';

type Props = {
  initial: MenuItem;
  onSave: (item: MenuItem) => void;
  onCancel: () => void;
};

export default function MenuItemForm({ initial, onSave, onCancel }: Props) {
  const [item, setItem] = useState<MenuItem>(initial);
  const appPath =
    item.action.kind === 'launch_app' ? item.action.path : undefined;

  const updateAction = (a: Action) => setItem({ ...item, action: a });

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
        <input
          type="text"
          placeholder="/Applications/Slack.app"
          className="border rounded px-2 py-1 w-full"
          value={item.action.path}
          onChange={(e) => updateAction({ kind: 'launch_app', path: e.target.value })}
        />
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
          <div className="space-y-2">
            <input
              type="text"
              placeholder="/usr/local/bin/foo"
              className="border rounded px-2 py-1 w-full"
              value={shell.command}
              onChange={(e) =>
                updateAction({ ...shell, command: e.target.value })
              }
            />
            <input
              type="text"
              placeholder="args (space-separated)"
              className="border rounded px-2 py-1 w-full"
              value={shell.args.join(' ')}
              onChange={(e) =>
                updateAction({
                  ...shell,
                  args: e.target.value.split(/\s+/).filter((a) => a.length > 0),
                })
              }
            />
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
