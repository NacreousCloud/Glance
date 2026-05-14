import { useEffect, useRef, useState } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import RingPulse from './RingPulse';
import IconBadge from './IconBadge';
import PersistentBadge from './PersistentBadge';

type Style = 'ring_pulse' | 'icon_badge' | 'persistent_badge';
type Payload = {
  id: string;
  timestamp_ms: number;
  style: Style;
  cursor_x: number;
  cursor_y: number;
  viewport_w: number;
  viewport_h: number;
  app_name: string;
  title: string;
  color_hue: number;
};

// Keep the indicator visually inside the viewport even if Rust-side window
// resize lags behind. Falls back to live window inner size when payload
// viewport is missing.
function clamp(payload: Payload): { x: number; y: number } {
  const safety = 48;
  const vw = payload.viewport_w || window.innerWidth;
  const vh = payload.viewport_h || window.innerHeight;
  const x = Math.max(safety, Math.min(payload.cursor_x, vw - safety));
  const y = Math.max(safety, Math.min(payload.cursor_y, vh - safety));
  if (x !== payload.cursor_x || y !== payload.cursor_y) {
    console.debug('[overlay] clamped cursor', {
      from: [payload.cursor_x, payload.cursor_y],
      to: [x, y],
      viewport: [vw, vh],
      windowInner: [window.innerWidth, window.innerHeight],
    });
  }
  return { x, y };
}

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
        const { x, y } = clamp(active);
        switch (active.style) {
          case 'ring_pulse':
            return <RingPulse key={active.id} x={x} y={y} hue={active.color_hue} />;
          case 'icon_badge':
            return (
              <IconBadge
                key={active.id}
                x={x}
                y={y}
                appName={active.app_name}
                hue={active.color_hue}
              />
            );
          case 'persistent_badge':
            return (
              <PersistentBadge
                key={active.id}
                x={x}
                y={y}
                appName={active.app_name}
                hue={active.color_hue}
              />
            );
          default:
            return null;
        }
      })}
    </>
  );
}

