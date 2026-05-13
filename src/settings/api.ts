import { invoke } from '@tauri-apps/api/core';

export type IndicatorStyle = 'ring_pulse' | 'icon_badge' | 'persistent_badge';
export type Settings = { indicator_style: IndicatorStyle; autostart: boolean };
export type PermissionStatus = {
  accessibility_ok: boolean;
  notification_listener_ok: boolean;
  platform: 'macos' | 'windows' | 'other';
};

export type NotiEvent = {
  id: number;
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
