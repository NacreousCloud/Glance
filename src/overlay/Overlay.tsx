import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';

type Payload = {
  style: 'ring_pulse' | 'icon_badge' | 'persistent_badge';
  cursor_x: number;
  cursor_y: number;
  app_name: string;
  title: string;
};

export default function Overlay() {
  const [active, setActive] = useState<Payload | null>(null);

  useEffect(() => {
    const un = listen<Payload>('noti:show', (e) => {
      setActive(e.payload);
      window.setTimeout(() => setActive(null), 1500);
    });
    return () => {
      un.then((f) => f());
    };
  }, []);

  if (!active) return null;
  return (
    <div style={{ position: 'fixed', inset: 0, pointerEvents: 'none' }}>
      <div
        style={{
          position: 'absolute',
          left: active.cursor_x - 24,
          top: active.cursor_y - 24,
          width: 48,
          height: 48,
          border: '2px solid #4ade80',
          borderRadius: '50%',
        }}
      />
    </div>
  );
}
