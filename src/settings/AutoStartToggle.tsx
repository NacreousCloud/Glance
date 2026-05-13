import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
import { useEffect, useState } from 'react';

export default function AutoStartToggle() {
  const [on, setOn] = useState<boolean | null>(null);
  useEffect(() => { isEnabled().then(setOn); }, []);

  const toggle = async () => {
    if (on) { await disable(); setOn(false); }
    else { await enable(); setOn(true); }
  };

  if (on === null) return null;
  return (
    <label className="flex items-center justify-between border rounded p-3">
      <span className="font-medium">Launch at login</span>
      <input type="checkbox" checked={on} onChange={toggle} />
    </label>
  );
}
