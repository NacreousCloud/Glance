import { act, renderHook, waitFor } from '@testing-library/react';
import { beforeEach, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { usePreferences } from './usePreferences';
import { DEFAULT_RADIAL_THEME } from './api';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
const initial = {
  indicator_style: 'ring_pulse', indicator_enabled: true, autostart: false,
  radial_close_on_leave: false, radial_theme: DEFAULT_RADIAL_THEME,
  menu_items: [{ id: 'old' }], hotkey_bindings: [{ id: 'old-key' }],
};
beforeEach(() => vi.mocked(invoke).mockReset());

it('saves only changed preferences, never stale menus or bindings', async () => {
  vi.mocked(invoke).mockImplementation(async (cmd) => cmd === 'get_settings' ? initial : undefined);
  const { result } = renderHook(usePreferences);
  await waitFor(() => expect(result.current.state).not.toBeNull());
  act(() => result.current.update({ indicator_enabled: false }));
  await waitFor(() => expect(invoke).toHaveBeenCalledWith('patch_preferences', {
    patch: { indicator_enabled: false },
  }));
  expect(invoke).not.toHaveBeenCalledWith('set_settings', expect.anything());
});

it('serializes saves and retries failed changes together with newer edits', async () => {
  let rejectFirst!: (e: Error) => void;
  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === 'get_settings') return initial;
    return new Promise((_, reject) => { rejectFirst = reject; });
  });
  const { result } = renderHook(usePreferences);
  await waitFor(() => expect(result.current.state).not.toBeNull());
  act(() => result.current.update({ indicator_enabled: false }));
  act(() => result.current.update({ radial_close_on_leave: true }));
  expect(vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === 'patch_preferences')).toHaveLength(1);
  await act(async () => rejectFirst(new Error('disk full')));
  expect(result.current.error).toContain('disk full');
  vi.mocked(invoke).mockResolvedValue(undefined);
  act(() => result.current.retry());
  await waitFor(() => expect(result.current.error).toBeNull());
  expect(invoke).toHaveBeenLastCalledWith('patch_preferences', {
    patch: { indicator_enabled: false, radial_close_on_leave: true },
  });
});
