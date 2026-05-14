type Props = { x: number; y: number; appName: string; hue: number };
export default function PersistentBadge({ x, y, appName, hue }: Props) {
  const bg = `hsl(${hue} 70% 35% / 0.9)`;
  return (
    <div
      data-testid="persistent-badge"
      style={{
        position: 'absolute',
        left: x,
        top: y,
        padding: '4px 8px',
        borderRadius: 12,
        background: bg,
        color: 'white',
        fontSize: 12,
        whiteSpace: 'nowrap',
      }}
    >
      {appName}
    </div>
  );
}
