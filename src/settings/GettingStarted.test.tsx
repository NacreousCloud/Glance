import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import GettingStarted from './GettingStarted';
import { DEFAULT_RADIAL_THEME } from './api';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
beforeEach(() => vi.mocked(invoke).mockReset());

it('dismisses the guide but keeps preview controls available', () => {
  const complete = vi.fn();
  const props = { completed: false, theme: DEFAULT_RADIAL_THEME, onComplete: complete, onConfigure: vi.fn() };
  const { rerender } = render(<GettingStarted {...props} />);
  fireEvent.click(screen.getByRole('button', { name: /finish setup/i }));
  expect(complete).toHaveBeenCalledOnce();
  rerender(<GettingStarted {...props} completed />);
  expect(screen.queryByText('Getting started')).not.toBeInTheDocument();
  expect(screen.getByRole('button', { name: /test notification/i })).toBeInTheDocument();
});

it('previews a shell item without invoking any action command', async () => {
  vi.mocked(invoke).mockResolvedValue([{
    id: 'shell', label: 'Shell preview', icon: { kind: 'emoji', value: 'X' },
    action: { kind: 'run_shell', command: 'dangerous', args: [], confirm: false }, tags: [],
  }]);
  render(<GettingStarted completed theme={DEFAULT_RADIAL_THEME} onComplete={vi.fn()} onConfigure={vi.fn()} />);
  fireEvent.click(screen.getByRole('button', { name: /preview menu/i }));
  fireEvent.mouseDown(await screen.findByText('Shell preview'));
  fireEvent.click(screen.getByText('Shell preview'));
  expect(invoke).toHaveBeenCalledTimes(1);
  expect(invoke).toHaveBeenCalledWith('list_menu_items');
});

it('reports test notification failure and allows retry', async () => {
  vi.mocked(invoke).mockRejectedValueOnce('overlay unavailable').mockResolvedValue(undefined);
  render(<GettingStarted completed theme={DEFAULT_RADIAL_THEME} onComplete={vi.fn()} onConfigure={vi.fn()} />);
  fireEvent.click(screen.getByRole('button', { name: /test notification/i }));
  expect(await screen.findByRole('alert')).toHaveTextContent('overlay unavailable');
  fireEvent.click(screen.getByRole('button', { name: /test notification/i }));
  await waitFor(() => expect(screen.queryByRole('alert')).not.toBeInTheDocument());
  expect(invoke).toHaveBeenLastCalledWith('test_notification');
});
