import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import RingPulse from './RingPulse';
import IconBadge from './IconBadge';
import PersistentBadge from './PersistentBadge';

describe('indicator styles', () => {
  it('RingPulse renders an svg ring centered at (x, y)', () => {
    render(<RingPulse x={100} y={150} />);
    const ring = screen.getByTestId('ring-pulse');
    expect(ring).toHaveAttribute('style', expect.stringContaining('left: 76px'));
    expect(ring).toHaveAttribute('style', expect.stringContaining('top: 126px'));
  });

  it('IconBadge renders app initial', () => {
    render(<IconBadge x={0} y={0} appName="Slack" />);
    expect(screen.getByText('S')).toBeInTheDocument();
  });

  it('PersistentBadge follows cursor when prop updates', () => {
    const { rerender } = render(<PersistentBadge x={10} y={20} appName="Mail" />);
    const el = screen.getByTestId('persistent-badge');
    expect(el.style.left).toBe('10px');
    rerender(<PersistentBadge x={50} y={60} appName="Mail" />);
    expect(el.style.left).toBe('50px');
  });
});
