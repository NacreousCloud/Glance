import { useEffect, useMemo, useRef, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { sectorAt } from './geometry';
import Sector from './Sector';
import { listMenuItems, execMenuItem } from './api';
import { getSettings } from '../settings/api';
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
  // Ignore mousedowns until this timestamp. Reset on every show event so a
  // phantom mousedown fired by macOS during window focus/show does not
  // immediately close the just-opened menu.
  const ignoreMouseUntil = useRef(0);

  const [closeOnLeave, setCloseOnLeave] = useState(false);

  useEffect(() => {
    listMenuItems().then(setItems);
    getSettings()
      .then((s) => setCloseOnLeave(!!s.radial_close_on_leave))
      .catch(() => {});
    const un = listen<ShowPayload>('radial:show', (e) => {
      listMenuItems().then(setItems);
      getSettings()
        .then((s) => setCloseOnLeave(!!s.radial_close_on_leave))
        .catch(() => {});
      setMenuMode(e.payload.menu_mode);
      setRecentAppName(e.payload.recent_app_name);
      // Short warm-up to absorb phantom mousedown from focus change.
      ignoreMouseUntil.current = Date.now() + 80;
    });
    const escListener = (ev: KeyboardEvent) => {
      if (ev.key === 'Escape') {
        invoke('hide_radial').catch(() => {});
      }
    };
    const blurListener = () => {
      invoke('hide_radial').catch(() => {});
    };
    window.addEventListener('keydown', escListener);
    window.addEventListener('blur', blurListener);
    return () => {
      un.then((f) => f());
      window.removeEventListener('keydown', escListener);
      window.removeEventListener('blur', blurListener);
    };
  }, []);

  const filtered = useMemo(
    () =>
      menuMode === 'all'
        ? items
        : items.filter((it) => it.tags.includes(menuMode)),
    [items, menuMode]
  );

  // Mirror closeOnLeave into a ref so the always-attached mouseleave
  // listener reads the latest value without re-attachment.
  const closeOnLeaveRef = useRef(false);
  useEffect(() => {
    closeOnLeaveRef.current = closeOnLeave;
  }, [closeOnLeave]);

  useEffect(() => {
    const maybeHide = () => {
      if (!closeOnLeaveRef.current) return;
      invoke('hide_radial').catch(() => {});
    };
    const onMouseOutWindow = (e: MouseEvent) => {
      if (e.relatedTarget !== null) return;
      maybeHide();
    };
    document.addEventListener('mouseleave', maybeHide);
    document.documentElement.addEventListener('mouseleave', maybeHide);
    document.body.addEventListener('mouseleave', maybeHide);
    document.addEventListener('mouseout', onMouseOutWindow);
    return () => {
      document.removeEventListener('mouseleave', maybeHide);
      document.documentElement.removeEventListener('mouseleave', maybeHide);
      document.body.removeEventListener('mouseleave', maybeHide);
      document.removeEventListener('mouseout', onMouseOutWindow);
    };
  }, []);

  // Window-level mousedown handler. SVG onClick does not fire reliably
  // through Tauri's transparent macOS window, so we listen at the document
  // level instead. Computes the sector from raw clientX/Y at click time.
  useEffect(() => {
    const onMouseDown = (e: MouseEvent) => {
      if (Date.now() < ignoreMouseUntil.current) return;
      const dx = e.clientX - CENTER;
      const dy = e.clientY - CENTER;
      const r = Math.sqrt(dx * dx + dy * dy);
      const sector = sectorAt(dx, dy, filtered.length, INNER, OUTER);
      if (r < INNER || sector === null) {
        invoke('hide_radial').catch(() => {});
        return;
      }
      const item = filtered[sector];
      if (item) execMenuItem(item.id);
      invoke('hide_radial').catch(() => {});
    };
    window.addEventListener('mousedown', onMouseDown);
    return () => window.removeEventListener('mousedown', onMouseDown);
  }, [filtered]);

  const onMouseMove = (e: React.MouseEvent<SVGSVGElement>) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const dx = e.clientX - rect.left - CENTER;
    const dy = e.clientY - rect.top - CENTER;
    const r = Math.sqrt(dx * dx + dy * dy);
    setCenterHovered(r < INNER);
    setHovered(sectorAt(dx, dy, filtered.length, INNER, OUTER));
  };

  return (
    <svg
      viewBox={`0 0 ${CENTER * 2} ${CENTER * 2}`}
      width={CENTER * 2}
      height={CENTER * 2}
      onMouseMove={onMouseMove}
      style={{ position: 'fixed', top: 0, left: 0, pointerEvents: 'auto' }}
    >
      {/* Semi-opaque backdrop. macOS transparent windows pass clicks
          through on alpha=0 areas, so a slight dim both gives a modal
          feel and ensures clicks in the 4 corners outside the pie are
          captured by the webview. */}
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
