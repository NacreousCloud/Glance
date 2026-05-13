type Props = { x: number; y: number };
export default function RingPulse({ x, y }: Props) {
  const size = 48;
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
        border: '3px solid #4ade80',
        animation: 'mn-pulse 900ms ease-out forwards',
      }}
    />
  );
}
