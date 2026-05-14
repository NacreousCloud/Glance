import type { MenuItem } from '../types';

type Props = {
  item: MenuItem;
  index: number;
  total: number;
  hovered: boolean;
  innerRadius: number;
  outerRadius: number;
};

export default function Sector({
  item,
  index,
  total,
  hovered,
  innerRadius,
  outerRadius,
}: Props) {
  const sectorAngle = (2 * Math.PI) / total;
  const startAngle = -Math.PI / 2 + index * sectorAngle;
  const endAngle = startAngle + sectorAngle;

  const x1 = Math.cos(startAngle) * outerRadius;
  const y1 = Math.sin(startAngle) * outerRadius;
  const x2 = Math.cos(endAngle) * outerRadius;
  const y2 = Math.sin(endAngle) * outerRadius;
  const x3 = Math.cos(endAngle) * innerRadius;
  const y3 = Math.sin(endAngle) * innerRadius;
  const x4 = Math.cos(startAngle) * innerRadius;
  const y4 = Math.sin(startAngle) * innerRadius;
  const largeArc = sectorAngle > Math.PI ? 1 : 0;

  const path = [
    `M ${x1} ${y1}`,
    `A ${outerRadius} ${outerRadius} 0 ${largeArc} 1 ${x2} ${y2}`,
    `L ${x3} ${y3}`,
    `A ${innerRadius} ${innerRadius} 0 ${largeArc} 0 ${x4} ${y4}`,
    'Z',
  ].join(' ');

  const iconAngle = (startAngle + endAngle) / 2;
  const iconR = (innerRadius + outerRadius) / 2;
  const iconX = Math.cos(iconAngle) * iconR;
  const iconY = Math.sin(iconAngle) * iconR;

  return (
    <g>
      <path
        d={path}
        fill={hovered ? 'rgba(96,165,250,0.85)' : 'rgba(31,41,55,0.85)'}
        stroke="rgba(255,255,255,0.2)"
        strokeWidth={1}
      />
      <foreignObject x={iconX - 20} y={iconY - 20} width={40} height={40} pointerEvents="none">
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', width: 40, height: 40, fontSize: 24 }}>
          {item.icon.kind === 'emoji' ? (
            <span>{item.icon.value}</span>
          ) : (
            <img src={`data:image/png;base64,${item.icon.base64}`} width={32} height={32} alt="" />
          )}
        </div>
      </foreignObject>
      <text x={iconX} y={iconY + 28} textAnchor="middle" fill="white" fontSize={10} pointerEvents="none">
        {item.label}
      </text>
    </g>
  );
}
