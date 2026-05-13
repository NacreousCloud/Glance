import { useEffect, useRef, useState } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import RingPulse from './RingPulse';
import IconBadge from './IconBadge';
import PersistentBadge from './PersistentBadge';

type Style = 'ring_pulse' | 'icon_badge' | 'persistent_badge';
type Payload = {
  id: number;
  style: Style;
  cursor_x: number;
  cursor_y: number;
  app_name: string;
  title: string;
};

export default function Overlay() {
  const [active, setActive] = useState<Payload | null>(null);
  const timer = useRef<number>();

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;

    const setup = async () => {
      unlisten = await listen<Payload>('noti:show', (e) => {
        window.clearTimeout(timer.current);
        setActive(e.payload);
        const duration = e.payload.style === 'persistent_badge' ? 5000 : 900;
        timer.current = window.setTimeout(() => setActive(null), duration);
      });
    };

    setup();

    return () => {
      if (unlisten) unlisten();
      window.clearTimeout(timer.current);
    };
  }, []);

  if (!active) return null;
  switch (active.style) {
    case 'ring_pulse':
      return (
        <RingPulse
          key={active.id}
          x={active.cursor_x}
          y={active.cursor_y}
        />
      );
    case 'icon_badge':
      return (
        <IconBadge
          key={active.id}
          x={active.cursor_x}
          y={active.cursor_y}
          appName={active.app_name}
        />
      );
    case 'persistent_badge':
      return (
        <PersistentBadge
          key={active.id}
          x={active.cursor_x}
          y={active.cursor_y}
          appName={active.app_name}
        />
      );
  }
}

