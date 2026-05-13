import { useEffect, useRef, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import RingPulse from './RingPulse';
import IconBadge from './IconBadge';
import PersistentBadge from './PersistentBadge';

type Style = 'ring_pulse' | 'icon_badge' | 'persistent_badge';
type Payload = { style: Style; cursor_x: number; cursor_y: number; app_name: string; title: string };

export default function Overlay() {
  const [active, setActive] = useState<Payload | null>(null);
  const timer = useRef<number | undefined>(undefined);

  useEffect(() => {
    const un = listen<Payload>('noti:show', (e) => {
      window.clearTimeout(timer.current);
      setActive(e.payload);
      const duration = e.payload.style === 'persistent_badge' ? 5000 : 900;
      timer.current = window.setTimeout(() => setActive(null), duration);
    });
    return () => {
      un.then((f) => f());
      window.clearTimeout(timer.current);
    };
  }, []);

  if (!active) return null;
  switch (active.style) {
    case 'ring_pulse':
      return <RingPulse x={active.cursor_x} y={active.cursor_y} />;
    case 'icon_badge':
      return <IconBadge x={active.cursor_x} y={active.cursor_y} appName={active.app_name} />;
    case 'persistent_badge':
      return <PersistentBadge x={active.cursor_x} y={active.cursor_y} appName={active.app_name} />;
  }
}
