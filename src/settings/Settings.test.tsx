import { render, screen, fireEvent } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import StylePicker from './StylePicker';

describe('StylePicker', () => {
  it('emits onChange when an option is clicked', () => {
    const onChange = vi.fn();
    render(<StylePicker value="ring_pulse" onChange={onChange} />);
    fireEvent.click(screen.getByLabelText(/icon badge/i));
    expect(onChange).toHaveBeenCalledWith('icon_badge');
  });

  it('marks current value as checked', () => {
    render(<StylePicker value="persistent_badge" onChange={() => {}} />);
    const radio = screen.getByLabelText(/persistent badge/i) as HTMLInputElement;
    expect(radio.checked).toBe(true);
  });
});
