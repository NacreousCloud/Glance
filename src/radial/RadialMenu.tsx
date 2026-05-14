import { useEffect, useMemo, useState } from 'react';
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
  const [menuMode, setMenuMode] = useState<string>('all');
  const [recentAppName, setRecentAppName] = useState<string | null>(null);

  useEffect(() => {
    listMenuItems().then(setItems);
    const un = listen<ShowPayload>('radial:show', (e) => {
      listMenuItems().then(setItems);
      setMenuMode(e.payload.menu_mode);
      setRecentAppName(e.payload.recent_app_name);
    });
    const escListener = (ev: KeyboardEvent) => {
      if (ev.key === 'Escape') {
        getCurrentWebviewWindow().hide();
      }
    };
    window.addEventListener('keydown', escListener);
    return () => {
      un.then((f) => f());
      window.removeEventListener('keydown', escListener);
    };
  }, []);

  const filtered = useMemo(
    () =>
      menuMode === 'all'
        ? items
        : items.filter((it) => it.tags.includes(menuMode)),
    [items, menuMode]
  );

  const onMouseMove = (e: React.MouseEvent<SVGSVGElement>) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const dx = e.clientX - rect.left - CENTER;
    const dy = e.clientY - rect.top - CENTER;
    setHovered(sectorAt(dx, dy, filtered.length, INNER, OUTER));
  };

  const onClick = () => {
    if (hovered === null) {
      getCurrentWebviewWindow().hide();
      return;
    }
    const item = filtered[hovered];
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
        {filtered.map((item, i) => (
          <Sector
            key={item.id}
            item={item}
            index={i}
            total={filtered.length}
            hovered={hovered === i}
            innerRadius={INNER}
            outerRadius={OUTER}
          />
        ))}
        {recentAppName && menuMode === 'notification' && (
          <text
            x={0}
            y={0}
            textAnchor="middle"
            dominantBaseline="middle"
            fill="white"
            fontSize={11}
            pointerEvents="none"
          >
            {recentAppName}
          </text>
        )}
      </g>
    </svg>
  );
}
