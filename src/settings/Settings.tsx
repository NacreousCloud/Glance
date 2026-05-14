import { useEffect, useState, useRef } from 'react';
import StylePicker from './StylePicker';
import PermissionPanel from './PermissionPanel';
import AutoStartToggle from './AutoStartToggle';
import NotificationLog from './NotificationLog';
import MenuEditor from './menu/MenuEditor';
import HotkeyEditor from './hotkey/HotkeyEditor';
import ErrorLog from './ErrorLog';
import { getSettings, setSettings, type Settings as S } from './api';

export default function Settings() {
  const [state, setState] = useState<S | null>(null);
  const saveTimer = useRef<number>();

  useEffect(() => {
    getSettings().then(setState);
    return () => window.clearTimeout(saveTimer.current);
  }, []);

  if (!state) return <div className="p-6">Loading…</div>;

  const update = (next: S) => {
    setState(next);
    // Debounce saves by 500ms
    window.clearTimeout(saveTimer.current);
    saveTimer.current = window.setTimeout(async () => {
      await setSettings(next);
    }, 500);
  };

  return (
    <div className="p-6 max-w-md mx-auto space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-xl font-bold">mouse-noti</h1>
        <span className="text-xs text-gray-400">v0.1.0</span>
      </div>
      <PermissionPanel />
      <hr className="border-gray-100" />
      <NotificationLog />
      <hr className="border-gray-100" />
      <StylePicker
        value={state.indicator_style}
        onChange={(s) => update({ ...state, indicator_style: s })}
      />
      <AutoStartToggle
        value={state.autostart}
        onChange={(v) => update({ ...state, autostart: v })}
      />
      <hr className="border-gray-100" />
      <MenuEditor />
      <hr className="border-gray-100" />
      <HotkeyEditor />
      <hr className="border-gray-100" />
      <ErrorLog />
      <hr className="border-gray-100" />
      <label className="flex items-center justify-between border rounded p-3">
        <div>
          <div className="font-medium">Close radial on cursor leave</div>
          <div className="text-xs text-gray-500">
            When on, the radial menu auto-closes the moment the cursor
            exits the menu window. Off (default): close requires explicit
            click, ESC, or focus loss.
          </div>
        </div>
        <input
          type="checkbox"
          checked={state.radial_close_on_leave}
          onChange={(e) =>
            update({ ...state, radial_close_on_leave: e.target.checked })
          }
        />
      </label>
    </div>
  );
}
