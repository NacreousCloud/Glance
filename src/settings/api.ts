import { invoke } from '@tauri-apps/api/core';
import type { HotkeyBinding, MenuItem } from '../types';

export type IndicatorStyle = 'ring_pulse' | 'icon_badge' | 'persistent_badge';
export type RadialTheme = {
  backdrop_color: string;
  backdrop_opacity: number;
  sector_color: string;
  sector_opacity: number;
  hover_color: string;
  center_color: string;
};
export const DEFAULT_RADIAL_THEME: RadialTheme = {
  backdrop_color: '#000000',
  backdrop_opacity: 0.0,
  sector_color: '#1f2937',
  sector_opacity: 0.85,
  hover_color: '#3b82f6',
  center_color: '#111827',
};
export type Settings = {
  indicator_style: IndicatorStyle;
  autostart: boolean;
  menu_items?: MenuItem[];
  hotkey_bindings?: HotkeyBinding[];
  radial_close_on_leave: boolean;
  radial_theme: RadialTheme;
};
export type PermissionStatus = {
  accessibility_ok: boolean;
  notification_listener_ok: boolean;
  platform: 'macos' | 'windows' | 'other';
};

export type NotiEvent = {
  id: string;
  timestamp_ms: number;
  app_id: string;
  app_name: string;
  title: string;
  body: string;
};

export const getSettings = () => invoke<Settings>('get_settings');
export const setSettings = (settings: Settings) => invoke<void>('set_settings', { settings });
export const permissionStatus = () => invoke<PermissionStatus>('permission_status');
export const requestPermission = () => invoke<void>('request_permission');
export const getRecentEvents = () => invoke<NotiEvent[]>('get_recent_events');

export const listMenuItems = () => invoke<MenuItem[]>('list_menu_items');
export const upsertMenuItem = (item: MenuItem) =>
  invoke<void>('upsert_menu_item', { item });
export const deleteMenuItem = (itemId: string) =>
  invoke<void>('delete_menu_item', { itemId });
export const reorderMenuItems = (ids: string[]) =>
  invoke<void>('reorder_menu_items', { ids });
export const extractAppIcon = (path: string) =>
  invoke<string>('extract_app_icon', { path });

export const listHotkeyBindings = () => invoke<HotkeyBinding[]>('list_hotkey_bindings');
export const upsertHotkeyBinding = (binding: HotkeyBinding) =>
  invoke<void>('upsert_hotkey_binding', { binding });
export const deleteHotkeyBinding = (bindingId: string) =>
  invoke<void>('delete_hotkey_binding', { bindingId });

export type ErrorEntry = {
  id: string;
  timestamp_ms: number;
  item_id: string;
  item_label: string;
  message: string;
};

export const getRecentErrors = () => invoke<ErrorEntry[]>('get_recent_errors');
export const clearErrors = () => invoke<void>('clear_errors');
