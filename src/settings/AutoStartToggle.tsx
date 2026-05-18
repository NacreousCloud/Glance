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
    <label className="ios-item ios-item-active">
      <div className="flex flex-col">
        <span className="ios-title">Launch at login</span>
        <span className="ios-subtitle">Start Glance automatically</span>
      </div>
      <input 
        type="checkbox" 
        className="w-11 h-6 appearance-none bg-gray-300 dark:bg-white/10 rounded-full relative transition-colors cursor-pointer checked:bg-ios-system-green before:content-[''] before:absolute before:w-5 before:h-5 before:bg-white before:rounded-full before:top-0.5 before:left-0.5 before:transition-transform checked:before:translate-x-5"
        checked={on} 
        onChange={toggle} 
      />
    </label>
  );
}
