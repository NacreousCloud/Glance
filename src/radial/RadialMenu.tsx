import { useEffect, useMemo, useRef, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { sectorAt } from './geometry';
import Sector from './Sector';
import { listMenuItems, execMenuItem } from './api';
import type { MenuItem } from '../types';

const radialLog = (msg: string) => {
  invoke<void>('radial_log', { msg })
    .then(() => {
      // Blue tint = invoke succeeded
      document.body.style.background = 'rgba(0, 0, 255, 0.35)';
    })
    .catch((err) => {
      // Red tint = invoke rejected
      document.body.style.background = 'rgba(255, 0, 0, 0.35)';
      document.title = `RADIAL_INVOKE_FAIL: ${String(err).slice(0, 80)}`;
    });
};

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
  // Ignore mousedowns until this timestamp. Reset on every show event so a
  // phantom mousedown fired by macOS during window focus/show does not
  // immediately close the just-opened menu.
  const ignoreMouseUntil = useRef(0);

  useEffect(() => {
    // VISUAL DIAGNOSTIC: green tint = RadialMenu mounted + useEffect ran.
    document.body.style.background = 'rgba(0, 255, 0, 0.35)';
    document.documentElement.style.background = 'rgba(0, 255, 0, 0.35)';
    document.title = `RADIAL_MOUNTED_${Date.now()}`;
    radialLog('first useEffect entered');
    listMenuItems().then(setItems);
    const un = listen<ShowPayload>('radial:show', (e) => {
      radialLog('radial:show event received in webview');
      listMenuItems().then(setItems);
      setMenuMode(e.payload.menu_mode);
      setRecentAppName(e.payload.recent_app_name);
      // Give the OS ~250ms to settle before we accept clicks. Phantom
      // mousedowns from focus changes / hotkey release should fall in
      // this window.
      ignoreMouseUntil.current = Date.now() + 250;
    });
    const escListener = (ev: KeyboardEvent) => {
      if (ev.key === 'Escape') {
        invoke('hide_radial').catch(() => {});
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

  // Window-level mousedown handler. SVG onClick was not firing reliably
  // through Tauri's transparent macOS window, so we listen at the document
  // level instead. Computes the sector from raw clientX/Y at click time.
  useEffect(() => {
    const onMouseDown = (e: MouseEvent) => {
      radialLog(`mousedown clientX=${e.clientX} clientY=${e.clientY} button=${e.button}`);
      if (Date.now() < ignoreMouseUntil.current) {
        radialLog('mousedown ignored (warm-up)');
        return;
      }
      const dx = e.clientX - CENTER;
      const dy = e.clientY - CENTER;
      const r = Math.sqrt(dx * dx + dy * dy);
      const sector = sectorAt(dx, dy, filtered.length, INNER, OUTER);
      radialLog(
        `mousedown decision r=${r.toFixed(1)} sector=${sector} items=${filtered.length}`
      );
      if (r < INNER || sector === null) {
        radialLog('mousedown → hide (center or outside)');
        invoke('hide_radial').catch(() => {});
        return;
      }
      const item = filtered[sector];
      if (item) {
        radialLog(`mousedown → exec item ${item.id}`);
        execMenuItem(item.id);
      }
      invoke('hide_radial').catch(() => {});
    };
    radialLog('mousedown listener installed');
    window.addEventListener('mousedown', onMouseDown);
    return () => {
      radialLog('mousedown listener removed');
      window.removeEventListener('mousedown', onMouseDown);
    };
  }, [filtered]);

  const onMouseMove = (e: React.MouseEvent<SVGSVGElement>) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const dx = e.clientX - rect.left - CENTER;
    const dy = e.clientY - rect.top - CENTER;
    const r = Math.sqrt(dx * dx + dy * dy);
    setCenterHovered(r < INNER);
    setHovered(sectorAt(dx, dy, filtered.length, INNER, OUTER));
  };

  // (Click handling moved to a window-level mousedown listener in useEffect
  // above so it works through Tauri's transparent macOS window.)

  return (
    <svg
      viewBox={`0 0 ${CENTER * 2} ${CENTER * 2}`}
      width={CENTER * 2}
      height={CENTER * 2}
      onMouseMove={onMouseMove}
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
