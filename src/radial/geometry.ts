export function sectorAt(
  dx: number,
  dy: number,
  n: number,
  innerRadius: number,
  outerRadius: number
): number | null {
  if (n === 0) return null;
  const r = Math.sqrt(dx * dx + dy * dy);
  if (r < innerRadius || r > outerRadius) return null;
  const thetaMath = Math.atan2(-dy, dx);
  const TAU = Math.PI * 2;
  let thetaCw = Math.PI / 2 - thetaMath;
  thetaCw = ((thetaCw % TAU) + TAU) % TAU;
  const sectorSize = TAU / n;
  return Math.floor(thetaCw / sectorSize) % n;
}
