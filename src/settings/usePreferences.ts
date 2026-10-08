import { useEffect, useRef, useState } from 'react';
import { getSettings, patchPreferences, type PreferencesPatch, type Settings } from './api';

export function usePreferences() {
  const [state, setState] = useState<Settings | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const pending = useRef<PreferencesPatch>({});
  const running = useRef(false);

  const load = () => {
    setError(null);
    getSettings().then(setState).catch((e) => setError(String(e)));
  };
  useEffect(load, []);

  const drain = async () => {
    if (running.current) return;
    running.current = true;
    setSaving(true);
    setError(null);
    try {
      while (Object.keys(pending.current).length) {
        const patch = pending.current;
        pending.current = {};
        try {
          await patchPreferences(patch);
        } catch (e) {
          pending.current = { ...patch, ...pending.current };
          setError(`Could not save settings: ${String(e)}`);
          break;
        }
      }
    } finally {
      running.current = false;
      setSaving(false);
    }
  };

  const update = (patch: PreferencesPatch) => {
    setState((current) => current ? { ...current, ...patch } : current);
    pending.current = { ...pending.current, ...patch };
    void drain();
  };

  return { state, error, saving, update, retry: () => state ? void drain() : load() };
}
