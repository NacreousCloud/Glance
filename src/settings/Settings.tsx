import { useEffect, useState } from 'react';
import StylePicker from './StylePicker';
import { getSettings, setSettings, type Settings as S } from './api';

export default function Settings() {
  const [state, setState] = useState<S | null>(null);

  useEffect(() => {
    getSettings().then(setState);
  }, []);

  if (!state) return <div className="p-6">Loading…</div>;

  const update = async (next: S) => {
    setState(next);
    await setSettings(next);
  };

  return (
    <div className="p-6 max-w-md mx-auto space-y-6">
      <h1 className="text-xl font-bold">mouse-noti</h1>
      <StylePicker
        value={state.indicator_style}
        onChange={(s) => update({ ...state, indicator_style: s })}
      />
    </div>
  );
}
