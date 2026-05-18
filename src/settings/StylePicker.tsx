import type { IndicatorStyle } from './api';

type Props = { value: IndicatorStyle; onChange: (v: IndicatorStyle) => void };

const OPTIONS: { value: IndicatorStyle; label: string; icon: string }[] = [
  { value: 'ring_pulse', label: 'Ring', icon: '○' },
  { value: 'icon_badge', label: 'Badge', icon: '⊡' },
  { value: 'persistent_badge', label: 'Dot', icon: '•' },
];

export default function StylePicker({ value, onChange }: Props) {
  return (
    <div className="flex gap-2">
      {OPTIONS.map((o) => (
        <button
          key={o.value}
          type="button"
          onClick={() => onChange(o.value)}
          className={`flex-1 flex flex-col items-center gap-1.5 p-3 rounded-ios border-2 transition-all ${
            value === o.value
              ? 'bg-ios-system-blue text-white border-ios-system-blue shadow-md'
              : 'bg-white/40 dark:bg-white/5 text-ios-label-secondary dark:text-ios-label-secondaryDark border-transparent hover:bg-white/60 dark:hover:bg-white/10'
          }`}
        >
          <span className="text-2xl leading-none h-6">{o.icon}</span>
          <span className="text-[11px] font-semibold uppercase tracking-tight">{o.label}</span>
        </button>
      ))}
    </div>
  );
}
