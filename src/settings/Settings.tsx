import { useEffect, useState, useRef } from 'react';
import StylePicker from './StylePicker';
import PermissionPanel from './PermissionPanel';
import AutoStartToggle from './AutoStartToggle';
import NotificationLog from './NotificationLog';
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
    </div>
  );
}
