import { useState } from 'react';
import Sector from '../radial/Sector';
import { hexToRgba } from '../radial/color';
import type { MenuItem } from '../types';
import type { RadialTheme } from './api';

// This component intentionally has no IPC/action imports. It draws the
// saved menu inside settings, independently of the executable radial window.
export default function RadialPreview({ items, theme }: { items: MenuItem[]; theme: RadialTheme }) {
  const [hovered, setHovered] = useState<number | null>(null);
  if (!items.length) return <p className="text-sm">Add a menu item in Radial Menu to preview it here.</p>;
  return <div>
    <p className="text-sm">Appearance preview — actions are disabled.</p>
    <svg viewBox="0 0 400 400" role="img" aria-label="Radial menu preview"
      className="w-full max-w-[400px] mx-auto">
      <rect width="400" height="400" fill={hexToRgba(theme.backdrop_color, theme.backdrop_opacity)} />
      <g transform="translate(200 200)">
        {items.map((item, index) => <g key={item.id}
          onMouseEnter={() => setHovered(index)} onMouseLeave={() => setHovered(null)}>
          <Sector item={item} index={index} total={items.length} hovered={hovered === index}
            innerRadius={28} outerRadius={180}
            baseFill={hexToRgba(theme.sector_color, theme.sector_opacity)}
            hoverFill={hexToRgba(theme.hover_color, 0.95)} />
        </g>)}
        <circle r="28" fill={hexToRgba(theme.center_color, 0.9)} />
      </g>
    </svg>
  </div>;
}
