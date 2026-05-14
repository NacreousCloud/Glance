import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { sectorAt } from './geometry';
import Sector from './Sector';
import { listMenuItems, execMenuItem } from './api';
import type { MenuItem } from '../types';

const CENTER = 200;
const INNER = 50;
const OUTER = 180;

type ShowPayload = {
  cursor_x: number;
  cursor_y: number;
  menu_mode: string;
  recent_app_name: string | null;
};

export default function RadialMenu() {
  const [items, setItems] = useState<MenuItem[]>([]);
  const [hovered, setHovered] = useState<number | null>(null);

  useEffect(() => {
    listMenuItems().then(setItems);
    const un = listen<ShowPayload>('radial:show', () => {
      listMenuItems().then(setItems);
    });
    const escListener = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        getCurrentWebviewWindow().hide();
      }
    };
    window.addEventListener('keydown', escListener);
    return () => {
      un.then((f) => f());
      window.removeEventListener('keydown', escListener);
    };
  }, []);

  const onMouseMove = (e: React.MouseEvent<SVGSVGElement>) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const dx = e.clientX - rect.left - CENTER;
    const dy = e.clientY - rect.top - CENTER;
    setHovered(sectorAt(dx, dy, items.length, INNER, OUTER));
  };

  const onClick = () => {
    if (hovered === null) {
      getCurrentWebviewWindow().hide();
      return;
    }
    const item = items[hovered];
    if (!item) return;
    execMenuItem(item.id);
    getCurrentWebviewWindow().hide();
  };

  return (
    <svg
      viewBox={`0 0 ${CENTER * 2} ${CENTER * 2}`}
      width={CENTER * 2}
      height={CENTER * 2}
      onMouseMove={onMouseMove}
      onClick={onClick}
      style={{ position: 'fixed', top: 0, left: 0 }}
    >
      <g transform={`translate(${CENTER} ${CENTER})`}>
        {items.map((item, i) => (
          <Sector
            key={item.id}
            item={item}
            index={i}
            total={items.length}
            hovered={hovered === i}
            innerRadius={INNER}
            outerRadius={OUTER}
          />
        ))}
      </g>
    </svg>
  );
}
