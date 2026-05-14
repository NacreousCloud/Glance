type Props = { x: number; y: number; hue: number };
export default function RingPulse({ x, y, hue }: Props) {
  const size = 48;
  const color = `hsl(${hue} 70% 55%)`;
  return (
    <div
      data-testid="ring-pulse"
      style={{
        position: 'absolute',
        left: x - size / 2,
        top: y - size / 2,
        width: size,
        height: size,
        borderRadius: '50%',
        border: `3px solid ${color}`,
        animation: 'mn-pulse 900ms ease-out forwards',
      }}
    />
  );
}
