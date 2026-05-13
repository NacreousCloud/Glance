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
  const [indicators, setIndicators] = useState<Payload[]>([]);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;

    const setup = async () => {
      unlisten = await listen<Payload>('noti:show', (e) => {
        const payload = e.payload;
        setIndicators((prev) => [...prev, payload]);

        const duration = payload.style === 'persistent_badge' ? 5000 : 900;
        window.setTimeout(() => {
          setIndicators((prev) => prev.filter((i) => i.id !== payload.id));
        }, duration);
      });
    };

    setup();

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  return (
    <>
      {indicators.map((active) => {
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
          default:
            return null;
        }
      })}
    </>
  );
}

