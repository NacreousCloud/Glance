import { useEffect, useMemo, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { sectorAt } from './geometry';
import Sector from './Sector';
import { listMenuItems, execMenuItem } from './api';
import type { MenuItem } from '../types';

const CENTER = 200;
// Pie design: wedges reach close to the center. INNER also acts as the
// hit-region radius for the central cancel button.
const INNER = 28;
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
  const [centerHovered, setCenterHovered] = useState(false);
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
    const r = Math.sqrt(dx * dx + dy * dy);
    setCenterHovered(r < INNER);
    setHovered(sectorAt(dx, dy, filtered.length, INNER, OUTER));
  };

  const onClick = (e: React.MouseEvent<SVGSVGElement>) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const dx = e.clientX - rect.left - CENTER;
    const dy = e.clientY - rect.top - CENTER;
    const r = Math.sqrt(dx * dx + dy * dy);
    const sector = sectorAt(dx, dy, filtered.length, INNER, OUTER);
    console.debug('[radial] click', { dx, dy, r, sector, centerHovered, hovered });
    // Center circle (r < INNER) or outside the pie (r > OUTER) → close.
    if (r < INNER || sector === null) {
      getCurrentWebviewWindow().hide();
      return;
    }
    const item = filtered[sector];
    if (!item) {
      getCurrentWebviewWindow().hide();
      return;
    }
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
      style={{ position: 'fixed', top: 0, left: 0, pointerEvents: 'auto' }}
    >
      {/* Semi-opaque backdrop. macOS transparent windows pass clicks through
          on alpha=0 areas, so a barely-visible alpha (e.g. 0.001) is still
          treated as transparent and clicks never reach onClick. A modest
          dim (0.15) gives a modal feel AND reliably captures clicks in
          the 4 corners outside the pie. */}
      <rect
        x={0}
        y={0}
        width={CENTER * 2}
        height={CENTER * 2}
        fill="rgba(0,0,0,0.18)"
      />
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
        {/* Central cancel button */}
        <circle
          r={INNER}
          fill={centerHovered ? 'rgba(239,68,68,0.9)' : 'rgba(17,24,39,0.9)'}
          stroke="rgba(255,255,255,0.25)"
          strokeWidth={1}
        />
        <text
          x={0}
          y={0}
          textAnchor="middle"
          dominantBaseline="middle"
          fill="white"
          fontSize={14}
          pointerEvents="none"
        >
          {centerHovered ? '×' : ''}
        </text>
        {recentAppName && menuMode === 'notification' && !centerHovered && (
          <text
            x={0}
            y={INNER + 14}
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
