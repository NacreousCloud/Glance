import { DEFAULT_RADIAL_THEME, type RadialTheme } from './api';

type Props = {
  value: RadialTheme;
  onChange: (next: RadialTheme) => void;
};

function ColorField({
  label,
  color,
  onColor,
  opacity,
  onOpacity,
}: {
  label: string;
  color: string;
  onColor: (v: string) => void;
  opacity?: number;
  onOpacity?: (v: number) => void;
}) {
  return (
    <div className="space-y-1 text-xs">
      <div className="flex items-center gap-2">
        <span className="flex-1 truncate text-gray-700">{label}</span>
        <input
          type="color"
          value={color}
          onChange={(e) => onColor(e.target.value)}
          className="h-6 w-8 shrink-0 cursor-pointer border-0 bg-transparent p-0"
        />
        <input
          type="text"
          value={color}
          onChange={(e) => onColor(e.target.value)}
          className="w-20 shrink-0 rounded border px-1 py-0.5 font-mono"
        />
      </div>
      {onOpacity !== undefined && opacity !== undefined ? (
        <div className="flex items-center gap-2 pl-2">
          <input
            type="range"
            min={0}
            max={100}
            value={Math.round(opacity * 100)}
            onChange={(e) => onOpacity(Number(e.target.value) / 100)}
            className="min-w-0 flex-1"
          />
          <span className="w-10 shrink-0 text-right tabular-nums text-gray-500">
            {Math.round(opacity * 100)}%
          </span>
        </div>
      ) : null}
    </div>
  );
}

export default function RadialThemeEditor({ value, onChange }: Props) {
  const set = (patch: Partial<RadialTheme>) => onChange({ ...value, ...patch });

  return (
    <section className="space-y-2">
      <div className="flex items-center justify-between">
        <h2 className="text-sm font-semibold">Radial appearance</h2>
        <button
          type="button"
          className="text-xs text-gray-500 hover:text-blue-600"
          onClick={() => onChange(DEFAULT_RADIAL_THEME)}
        >
          Reset
        </button>
      </div>
      <div className="space-y-2 rounded border p-3">
        <ColorField
          label="Backdrop"
          color={value.backdrop_color}
          onColor={(v) => set({ backdrop_color: v })}
          opacity={value.backdrop_opacity}
          onOpacity={(v) => set({ backdrop_opacity: v })}
        />
        <ColorField
          label="Sector"
          color={value.sector_color}
          onColor={(v) => set({ sector_color: v })}
          opacity={value.sector_opacity}
          onOpacity={(v) => set({ sector_opacity: v })}
        />
        <ColorField
          label="Hover"
          color={value.hover_color}
          onColor={(v) => set({ hover_color: v })}
        />
        <ColorField
          label="Center disc"
          color={value.center_color}
          onColor={(v) => set({ center_color: v })}
        />
        <p className="text-xs text-gray-400">
          Backdrop opacity 0% → square shape disappears (click outside still
          closes the menu via window blur).
        </p>
      </div>
    </section>
  );
}
