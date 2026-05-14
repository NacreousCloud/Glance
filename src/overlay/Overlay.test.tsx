import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import RingPulse from './RingPulse';
import IconBadge from './IconBadge';
import PersistentBadge from './PersistentBadge';

describe('indicator styles', () => {
  it('RingPulse renders an svg ring centered at (x, y)', () => {
    render(<RingPulse x={100} y={150} hue={120} />);
    const ring = screen.getByTestId('ring-pulse');
    expect(ring).toHaveAttribute('style', expect.stringContaining('left: 76px'));
    expect(ring).toHaveAttribute('style', expect.stringContaining('top: 126px'));
  });

  it('IconBadge renders app initial', () => {
    render(<IconBadge x={0} y={0} appName="Slack" hue={210} />);
    expect(screen.getByText('S')).toBeInTheDocument();
  });

  it('PersistentBadge follows cursor when prop updates', () => {
    const { rerender } = render(<PersistentBadge x={10} y={20} appName="Mail" hue={45} />);
    const el = screen.getByTestId('persistent-badge');
    expect(el.style.left).toBe('10px');
    rerender(<PersistentBadge x={50} y={60} appName="Mail" hue={45} />);
    expect(el.style.left).toBe('50px');
  });

  it('RingPulse uses hue in border color', () => {
    // JSDOM normalizes inline hsl() to rgb(); assert hue actually changes the
    // rendered color rather than matching the raw hsl() literal.
    const { rerender, container } = render(<RingPulse x={0} y={0} hue={210} />);
    const ring = container.querySelector('[data-testid="ring-pulse"]') as HTMLElement;
    const colorAt210 = ring.style.borderColor;
    expect(colorAt210).not.toBe('');
    rerender(<RingPulse x={0} y={0} hue={30} />);
    const colorAt30 = ring.style.borderColor;
    expect(colorAt30).not.toBe(colorAt210);
  });
});
