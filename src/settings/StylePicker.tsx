import type { IndicatorStyle } from './api';

type Props = { value: IndicatorStyle; onChange: (v: IndicatorStyle) => void };

const OPTIONS: { value: IndicatorStyle; label: string }[] = [
  { value: 'ring_pulse', label: 'Ring Pulse' },
  { value: 'icon_badge', label: 'Icon Badge' },
  { value: 'persistent_badge', label: 'Persistent Badge' },
];

export default function StylePicker({ value, onChange }: Props) {
  return (
    <fieldset className="space-y-2">
      <legend className="text-sm font-semibold mb-2">Indicator style</legend>
      {OPTIONS.map((o) => (
        <label key={o.value} className="flex items-center gap-2 cursor-pointer">
          <input
            type="radio"
            name="indicator-style"
            value={o.value}
            checked={value === o.value}
            onChange={() => onChange(o.value)}
          />
          {o.label}
        </label>
      ))}
    </fieldset>
  );
}
