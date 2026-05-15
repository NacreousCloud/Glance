import { useEffect, useState, useRef } from 'react';
import StylePicker from './StylePicker';
import PermissionPanel from './PermissionPanel';
import AutoStartToggle from './AutoStartToggle';
import NotificationLog from './NotificationLog';
import MenuEditor from './menu/MenuEditor';
import HotkeyEditor from './hotkey/HotkeyEditor';
import ErrorLog from './ErrorLog';
import RadialThemeEditor from './RadialThemeEditor';
import { getSettings, setSettings, DEFAULT_RADIAL_THEME, type Settings as S } from './api';

export default function Settings() {
  const [state, setState] = useState<S | null>(null);
  const saveTimer = useRef<number | undefined>(undefined);

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
        <h1 className="text-xl font-bold">Glance</h1>
        <span className="text-xs text-gray-400">v0.6.1</span>
      </div>
      <PermissionPanel />
      <hr className="border-gray-100" />
      <NotificationLog />
      <hr className="border-gray-100" />
      <label className="flex items-center justify-between border rounded p-3">
        <div>
          <div className="font-medium">Notification indicator</div>
          <div className="text-xs text-gray-500">
            Master switch. Off = no overlay on system notifications (error
            badges from menu actions still show).
          </div>
        </div>
        <input
          type="checkbox"
          checked={state.indicator_enabled ?? true}
          onChange={(e) =>
            update({ ...state, indicator_enabled: e.target.checked })
          }
        />
      </label>
      <StylePicker
        value={state.indicator_style}
        onChange={(s) => update({ ...state, indicator_style: s })}
      />
      <AutoStartToggle />
      <hr className="border-gray-100" />
      <MenuEditor />
      <hr className="border-gray-100" />
      <HotkeyEditor />
      <hr className="border-gray-100" />
      <ErrorLog />
      <hr className="border-gray-100" />
      <RadialThemeEditor
        value={state.radial_theme ?? DEFAULT_RADIAL_THEME}
        onChange={(t) => update({ ...state, radial_theme: t })}
      />
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
