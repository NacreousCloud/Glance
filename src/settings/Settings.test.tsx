import { render, screen, fireEvent } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import StylePicker from './StylePicker';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (cmd: string) => {
    if (cmd === 'permission_status') return { accessibility_ok: false, notification_listener_ok: false, platform: 'macos' };
    if (cmd === 'request_permission') return undefined;
    return undefined;
  }),
}));

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

describe('PermissionPanel', () => {
  it('shows Required + Grant button when accessibility not granted on macOS', async () => {
    const { default: PermissionPanel } = await import('./PermissionPanel');
    render(<PermissionPanel />);
    expect(await screen.findByText('Required')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /grant/i })).toBeInTheDocument();
  });
});
