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
    <div className="p-3 space-y-2">
      <div className="flex items-center justify-between">
        <span className="ios-title">{label}</span>
        <div className="flex items-center gap-2">
          <input
            type="text"
            value={color.toUpperCase()}
            onChange={(e) => onColor(e.target.value)}
            className="w-20 bg-gray-100 dark:bg-white/10 rounded px-2 py-1 font-mono text-[13px] text-center outline-none"
          />
          <div className="relative w-8 h-8 rounded-full border-2 border-white shadow-sm overflow-hidden" style={{ backgroundColor: color }}>
            <input
              type="color"
              value={color}
              onChange={(e) => onColor(e.target.value)}
              className="absolute inset-0 opacity-0 w-full h-full cursor-pointer"
            />
          </div>
        </div>
      </div>
      {onOpacity !== undefined && opacity !== undefined && (
        <div className="flex items-center gap-3 px-1">
          <input
            type="range"
            min={0}
            max={100}
            value={Math.round(opacity * 100)}
            onChange={(e) => onOpacity(Number(e.target.value) / 100)}
            className="flex-1 accent-ios-system-blue h-1.5 bg-gray-200 dark:bg-white/10 rounded-full appearance-none"
          />
          <span className="w-10 text-right tabular-nums text-[13px] text-ios-label-secondary font-medium">
            {Math.round(opacity * 100)}%
          </span>
        </div>
      )}
    </div>
  );
}

export default function RadialThemeEditor({ value, onChange }: Props) {
  const set = (patch: Partial<RadialTheme>) => onChange({ ...value, ...patch });

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between px-1">
        <h2 className="text-[13px] uppercase tracking-wider text-ios-label-secondary dark:text-ios-label-secondaryDark">
          Radial Appearance
        </h2>
        <button
          type="button"
          className="text-[13px] font-medium text-ios-system-blue active:opacity-60"
          onClick={() => onChange(DEFAULT_RADIAL_THEME)}
        >
          Reset Theme
        </button>
      </div>

      <div className="ios-card divide-y divide-ios-separator-light dark:divide-ios-separator-dark">
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
          label="Hover State"
          color={value.hover_color}
          onColor={(v) => set({ hover_color: v })}
        />
        <ColorField
          label="Center Circle"
          color={value.center_color}
          onColor={(v) => set({ center_color: v })}
        />
      </div>
      <p className="px-4 text-[12px] text-ios-label-secondary dark:text-ios-label-secondaryDark italic leading-snug">
        Tip: Set Backdrop opacity to 0% for a clean circular look.
      </p>
    </div>
  );
}
