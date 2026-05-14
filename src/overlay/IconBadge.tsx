type Props = { x: number; y: number; appName: string; hue: number };
export default function IconBadge({ x, y, appName, hue }: Props) {
  const initial = appName.trim().slice(0, 1).toUpperCase() || '?';
  const bg = `hsl(${hue} 70% 45%)`;
  return (
    <div
      data-testid="icon-badge"
      style={{
        position: 'absolute',
        left: x + 12,
        top: y + 12,
        width: 28,
        height: 28,
        borderRadius: 8,
        background: bg,
        color: 'white',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontWeight: 700,
        boxShadow: '0 2px 6px rgba(0,0,0,0.3)',
      }}
    >
      {initial}
    </div>
  );
}
