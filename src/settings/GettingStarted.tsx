import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listMenuItems, type RadialTheme } from './api';
import type { MenuItem } from '../types';
import RadialPreview from './RadialPreview';

type Props = {
  completed: boolean;
  theme: RadialTheme;
  saving?: boolean;
  onComplete: () => void;
  onConfigure: () => void;
};

export default function GettingStarted({ completed, theme, saving, onComplete, onConfigure }: Props) {
  const [items, setItems] = useState<MenuItem[] | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const run = async (action: () => Promise<void>) => {
    setBusy(true); setError(null); setMessage(null);
    try { await action(); } catch (e) { setError(String(e)); }
    finally { setBusy(false); }
  };
  const buttonClass = 'px-3 py-2 rounded-lg bg-ios-system-blue text-white text-sm disabled:opacity-50';

  return <section className="ios-card p-4 space-y-3" aria-label="Setup and previews">
    <h2 className="font-semibold">{completed ? 'Try Glance' : 'Getting started'}</h2>
    {!completed && <>
      <ol className="list-decimal pl-5 space-y-1 text-sm">
        <li>Check Permissions below for available features.</li>
        <li>Test an indicator at your cursor. This works without OS notification access, even when indicators are switched off.</li>
        <li>Add menu items and a hotkey in Radial Menu, then try your hotkey.</li>
      </ol>
      <p className="text-sm">Glance stays in the menu bar or system tray. Open Settings there anytime.</p>
    </>}
    <div className="flex flex-wrap gap-2">
      <button type="button" className={buttonClass} disabled={busy || saving}
        onClick={() => void run(async () => {
          await invoke('test_notification');
          setMessage('Test notification sent to your cursor.');
        })}>Test notification</button>
      <button type="button" className={buttonClass} disabled={busy}
        onClick={() => void run(async () => setItems(await listMenuItems()))}>Preview menu</button>
      {!completed && <button type="button" className={buttonClass} onClick={onConfigure}>Configure menu & hotkeys</button>}
    </div>
    {error && <p role="alert" className="text-ios-system-red text-sm">{error}</p>}
    {message && <p role="status" className="text-sm">{message}</p>}
    {items !== null && <>
      <RadialPreview items={items} theme={theme} />
      <button type="button" className="text-sm underline" onClick={() => setItems(null)}>Hide preview</button>
    </>}
    {!completed && <button type="button" className="text-sm underline" disabled={saving} onClick={onComplete}>Finish setup</button>}
  </section>;
}
