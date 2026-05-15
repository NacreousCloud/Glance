export type IndicatorStyle = 'ring_pulse' | 'icon_badge' | 'persistent_badge';

export type IconSource =
  | { kind: 'emoji'; value: string }
  | { kind: 'app_icon_png'; base64: string; source_path: string };

export type Action =
  | { kind: 'launch_app'; path: string }
  | { kind: 'open_url'; url: string }
  | { kind: 'run_shell'; command: string; args: string[]; confirm: boolean };

export type MenuItem = {
  id: string;
  label: string;
  icon: IconSource;
  action: Action;
  tags: string[];
};

export type Corner = 'top_left' | 'top_right' | 'bottom_left' | 'bottom_right';

export type HotkeyTrigger =
  | { kind: 'keyboard'; accelerator: string }
  | { kind: 'mouse'; button: number; modifiers: number }
  | { kind: 'force_touch' }
  | { kind: 'hot_corner'; corner: Corner; radius_px: number };

export type HotkeyBinding = {
  id: string;
  trigger: HotkeyTrigger;
  menu_mode: string;
};
