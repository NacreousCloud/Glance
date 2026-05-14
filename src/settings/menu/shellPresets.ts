export type ShellPlatform = 'macos' | 'windows';

export type ShellPreset = {
  id: string;
  label: string;
  icon: string; // emoji
  command: string;
  args: string[];
  confirm: boolean;
  description?: string;
  platform: ShellPlatform;
};

export function detectPlatform(): ShellPlatform | 'other' {
  if (typeof navigator === 'undefined') return 'other';
  const ua = navigator.userAgent;
  if (ua.includes('Mac OS X') || ua.includes('Macintosh')) return 'macos';
  if (ua.includes('Windows')) return 'windows';
  return 'other';
}

export function presetsForCurrentPlatform(): ShellPreset[] {
  const p = detectPlatform();
  if (p === 'other') return [];
  return SHELL_PRESETS.filter((x) => x.platform === p);
}

// Curated shell-command presets. Each is safe enough to default
// confirm=false. Anything destructive or write-heavy keeps confirm=true.
export const SHELL_PRESETS: ShellPreset[] = [
  // ─── macOS ────────────────────────────────────────────────────────────
  {
    id: 'mac-toggle-dark-mode',
    label: 'Toggle Dark Mode',
    icon: '🌙',
    command: '/usr/bin/osascript',
    args: [
      '-e',
      'tell app "System Events" to tell appearance preferences to set dark mode to not dark mode',
    ],
    confirm: false,
    platform: 'macos',
  },
  {
    id: 'mac-lock-screen',
    label: 'Lock Screen',
    icon: '🔒',
    command: '/usr/bin/pmset',
    args: ['displaysleepnow'],
    confirm: false,
    platform: 'macos',
  },
  {
    id: 'mac-sleep',
    label: 'Sleep',
    icon: '💤',
    command: '/usr/bin/pmset',
    args: ['sleepnow'],
    confirm: true,
    platform: 'macos',
  },
  {
    id: 'mac-screenshot-area',
    label: 'Screenshot (area)',
    icon: '📸',
    command: '/bin/sh',
    args: [
      '-c',
      'screencapture -i ~/Desktop/Screenshot-$(date +%Y%m%d-%H%M%S).png',
    ],
    confirm: false,
    description:
      'Requires Screen Recording permission for Glance (System Settings → Privacy & Security → Screen Recording). Without it, only the desktop wallpaper is captured — windows appear blank.',
    platform: 'macos',
  },
  {
    id: 'mac-music-playpause',
    label: 'Music Play/Pause',
    icon: '▶️',
    command: '/usr/bin/osascript',
    args: ['-e', 'tell application "Music" to playpause'],
    confirm: false,
    platform: 'macos',
  },
  {
    id: 'mac-music-next',
    label: 'Music Next',
    icon: '⏭️',
    command: '/usr/bin/osascript',
    args: ['-e', 'tell application "Music" to next track'],
    confirm: false,
    platform: 'macos',
  },
  {
    id: 'mac-music-previous',
    label: 'Music Previous',
    icon: '⏮️',
    command: '/usr/bin/osascript',
    args: ['-e', 'tell application "Music" to previous track'],
    confirm: false,
    platform: 'macos',
  },
  {
    id: 'mac-volume-50',
    label: 'Volume 50%',
    icon: '🔊',
    command: '/usr/bin/osascript',
    args: ['-e', 'set volume output volume 50'],
    confirm: false,
    platform: 'macos',
  },
  {
    id: 'mac-mute',
    label: 'Mute',
    icon: '🔇',
    command: '/usr/bin/osascript',
    args: ['-e', 'set volume output muted true'],
    confirm: false,
    platform: 'macos',
  },
  {
    id: 'mac-open-downloads',
    label: 'Open Downloads',
    icon: '📁',
    command: '/bin/sh',
    args: ['-c', 'open ~/Downloads'],
    confirm: false,
    platform: 'macos',
  },
  {
    id: 'mac-open-documents',
    label: 'Open Documents',
    icon: '📂',
    command: '/bin/sh',
    args: ['-c', 'open ~/Documents'],
    confirm: false,
    platform: 'macos',
  },
  {
    id: 'mac-open-home',
    label: 'Open Home',
    icon: '🏠',
    command: '/bin/sh',
    args: ['-c', 'open ~'],
    confirm: false,
    platform: 'macos',
  },
  {
    id: 'mac-mission-control',
    label: 'Mission Control',
    icon: '🚀',
    command: '/usr/bin/open',
    args: ['-a', 'Mission Control'],
    confirm: false,
    platform: 'macos',
  },
  {
    id: 'mac-wifi-off',
    label: 'Wi-Fi Off',
    icon: '📡',
    command: '/usr/sbin/networksetup',
    args: ['-setairportpower', 'en0', 'off'],
    confirm: true,
    platform: 'macos',
  },
  {
    id: 'mac-wifi-on',
    label: 'Wi-Fi On',
    icon: '📡',
    command: '/usr/sbin/networksetup',
    args: ['-setairportpower', 'en0', 'on'],
    confirm: false,
    platform: 'macos',
  },
  {
    id: 'mac-empty-trash',
    label: 'Empty Trash',
    icon: '🗑️',
    command: '/usr/bin/osascript',
    args: ['-e', 'tell application "Finder" to empty trash'],
    confirm: true,
    platform: 'macos',
  },

  // ─── Windows ──────────────────────────────────────────────────────────
  {
    id: 'win-lock-screen',
    label: 'Lock Screen',
    icon: '🔒',
    command: 'rundll32.exe',
    args: ['user32.dll,LockWorkStation'],
    confirm: false,
    platform: 'windows',
  },
  {
    id: 'win-sleep',
    label: 'Sleep',
    icon: '💤',
    command: 'rundll32.exe',
    args: ['powrprof.dll,SetSuspendState', '0,1,0'],
    confirm: true,
    description:
      'Sleeps if hibernation is disabled; otherwise hibernates. Use `powercfg /h off` once to force sleep behavior.',
    platform: 'windows',
  },
  {
    id: 'win-screenshot-area',
    label: 'Screenshot (area)',
    icon: '📸',
    command: 'cmd',
    args: ['/c', 'start', 'ms-screenclip:'],
    confirm: false,
    description:
      'Opens the Snip & Sketch area selector. Captured image is placed on the clipboard.',
    platform: 'windows',
  },
  {
    id: 'win-media-playpause',
    label: 'Media Play/Pause',
    icon: '▶️',
    command: 'powershell',
    args: [
      '-NoProfile',
      '-Command',
      '(New-Object -ComObject WScript.Shell).SendKeys([char]179)',
    ],
    confirm: false,
    platform: 'windows',
  },
  {
    id: 'win-media-next',
    label: 'Media Next',
    icon: '⏭️',
    command: 'powershell',
    args: [
      '-NoProfile',
      '-Command',
      '(New-Object -ComObject WScript.Shell).SendKeys([char]176)',
    ],
    confirm: false,
    platform: 'windows',
  },
  {
    id: 'win-media-previous',
    label: 'Media Previous',
    icon: '⏮️',
    command: 'powershell',
    args: [
      '-NoProfile',
      '-Command',
      '(New-Object -ComObject WScript.Shell).SendKeys([char]177)',
    ],
    confirm: false,
    platform: 'windows',
  },
  {
    id: 'win-mute',
    label: 'Mute',
    icon: '🔇',
    command: 'powershell',
    args: [
      '-NoProfile',
      '-Command',
      '(New-Object -ComObject WScript.Shell).SendKeys([char]173)',
    ],
    confirm: false,
    platform: 'windows',
  },
  {
    id: 'win-volume-up',
    label: 'Volume +',
    icon: '🔊',
    command: 'powershell',
    args: [
      '-NoProfile',
      '-Command',
      '$s=New-Object -ComObject WScript.Shell;1..5|%{$s.SendKeys([char]175)}',
    ],
    confirm: false,
    platform: 'windows',
  },
  {
    id: 'win-volume-down',
    label: 'Volume -',
    icon: '🔉',
    command: 'powershell',
    args: [
      '-NoProfile',
      '-Command',
      '$s=New-Object -ComObject WScript.Shell;1..5|%{$s.SendKeys([char]174)}',
    ],
    confirm: false,
    platform: 'windows',
  },
  {
    id: 'win-open-downloads',
    label: 'Open Downloads',
    icon: '📁',
    command: 'explorer',
    args: ['%USERPROFILE%\\Downloads'],
    confirm: false,
    platform: 'windows',
  },
  {
    id: 'win-open-documents',
    label: 'Open Documents',
    icon: '📂',
    command: 'explorer',
    args: ['%USERPROFILE%\\Documents'],
    confirm: false,
    platform: 'windows',
  },
  {
    id: 'win-open-home',
    label: 'Open Home',
    icon: '🏠',
    command: 'explorer',
    args: ['%USERPROFILE%'],
    confirm: false,
    platform: 'windows',
  },
  {
    id: 'win-task-manager',
    label: 'Task Manager',
    icon: '🛠️',
    command: 'taskmgr',
    args: [],
    confirm: false,
    platform: 'windows',
  },
  {
    id: 'win-empty-recycle',
    label: 'Empty Recycle Bin',
    icon: '🗑️',
    command: 'powershell',
    args: ['-NoProfile', '-Command', 'Clear-RecycleBin -Force'],
    confirm: true,
    platform: 'windows',
  },
  {
    id: 'win-wifi-off',
    label: 'Wi-Fi Off',
    icon: '📡',
    command: 'netsh',
    args: ['interface', 'set', 'interface', 'Wi-Fi', 'admin=disable'],
    confirm: true,
    description: 'Needs administrator privileges.',
    platform: 'windows',
  },
  {
    id: 'win-wifi-on',
    label: 'Wi-Fi On',
    icon: '📡',
    command: 'netsh',
    args: ['interface', 'set', 'interface', 'Wi-Fi', 'admin=enable'],
    confirm: false,
    description: 'Needs administrator privileges.',
    platform: 'windows',
  },
];
