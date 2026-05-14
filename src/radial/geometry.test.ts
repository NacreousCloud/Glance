import { describe, it, expect } from 'vitest';
import { sectorAt } from './geometry';

describe('sectorAt', () => {
  it('top is sector 0 for n=4', () => {
    expect(sectorAt(0, -100, 4, 20, 200)).toBe(0);
  });
  it('right is sector 1 for n=4', () => {
    expect(sectorAt(100, 0, 4, 20, 200)).toBe(1);
  });
  it('inner circle returns null', () => {
    expect(sectorAt(5, 5, 6, 20, 200)).toBeNull();
  });
});
