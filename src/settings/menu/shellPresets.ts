export type ShellPreset = {
  id: string;
  label: string;
  icon: string; // emoji
  command: string;
  args: string[];
  confirm: boolean;
  description?: string;
};

// Curated macOS shell-command presets. Each is safe enough to default
// confirm=false. Anything destructive or write-heavy keeps confirm=true.
export const SHELL_PRESETS: ShellPreset[] = [
  {
    id: 'toggle-dark-mode',
    label: 'Toggle Dark Mode',
    icon: '🌙',
    command: '/usr/bin/osascript',
    args: [
      '-e',
      'tell app "System Events" to tell appearance preferences to set dark mode to not dark mode',
    ],
    confirm: false,
  },
  {
    id: 'lock-screen',
    label: 'Lock Screen',
    icon: '🔒',
    command: '/usr/bin/pmset',
    args: ['displaysleepnow'],
    confirm: false,
  },
  {
    id: 'sleep',
    label: 'Sleep',
    icon: '💤',
    command: '/usr/bin/pmset',
    args: ['sleepnow'],
    confirm: true,
  },
  {
    id: 'screenshot-area',
    label: 'Screenshot (area)',
    icon: '📸',
    command: '/bin/sh',
    args: [
      '-c',
      'screencapture -i ~/Desktop/Screenshot-$(date +%Y%m%d-%H%M%S).png',
    ],
    confirm: false,
  },
  {
    id: 'music-playpause',
    label: 'Music Play/Pause',
    icon: '▶️',
    command: '/usr/bin/osascript',
    args: ['-e', 'tell application "Music" to playpause'],
    confirm: false,
  },
  {
    id: 'music-next',
    label: 'Music Next',
    icon: '⏭️',
    command: '/usr/bin/osascript',
    args: ['-e', 'tell application "Music" to next track'],
    confirm: false,
  },
  {
    id: 'music-previous',
    label: 'Music Previous',
    icon: '⏮️',
    command: '/usr/bin/osascript',
    args: ['-e', 'tell application "Music" to previous track'],
    confirm: false,
  },
  {
    id: 'volume-50',
    label: 'Volume 50%',
    icon: '🔊',
    command: '/usr/bin/osascript',
    args: ['-e', 'set volume output volume 50'],
    confirm: false,
  },
  {
    id: 'mute',
    label: 'Mute',
    icon: '🔇',
    command: '/usr/bin/osascript',
    args: ['-e', 'set volume output muted true'],
    confirm: false,
  },
  {
    id: 'open-downloads',
    label: 'Open Downloads',
    icon: '📁',
    command: '/bin/sh',
    args: ['-c', 'open ~/Downloads'],
    confirm: false,
  },
  {
    id: 'open-documents',
    label: 'Open Documents',
    icon: '📂',
    command: '/bin/sh',
    args: ['-c', 'open ~/Documents'],
    confirm: false,
  },
  {
    id: 'open-home',
    label: 'Open Home',
    icon: '🏠',
    command: '/bin/sh',
    args: ['-c', 'open ~'],
    confirm: false,
  },
  {
    id: 'mission-control',
    label: 'Mission Control',
    icon: '🚀',
    command: '/usr/bin/open',
    args: ['-a', 'Mission Control'],
    confirm: false,
  },
  {
    id: 'wifi-off',
    label: 'Wi-Fi Off',
    icon: '📡',
    command: '/usr/sbin/networksetup',
    args: ['-setairportpower', 'en0', 'off'],
    confirm: true,
  },
  {
    id: 'wifi-on',
    label: 'Wi-Fi On',
    icon: '📡',
    command: '/usr/sbin/networksetup',
    args: ['-setairportpower', 'en0', 'on'],
    confirm: false,
  },
  {
    id: 'empty-trash',
    label: 'Empty Trash',
    icon: '🗑️',
    command: '/usr/bin/osascript',
    args: ['-e', 'tell application "Finder" to empty trash'],
    confirm: true,
  },
];
