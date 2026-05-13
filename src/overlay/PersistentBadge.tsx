type Props = { x: number; y: number; appName: string };
export default function PersistentBadge({ x, y, appName }: Props) {
  return (
    <div
      data-testid="persistent-badge"
      style={{
        position: 'absolute',
        left: x,
        top: y,
        padding: '4px 8px',
        borderRadius: 12,
        background: 'rgba(31,41,55,0.9)',
        color: 'white',
        fontSize: 12,
        whiteSpace: 'nowrap',
      }}
    >
      {appName}
    </div>
  );
}
