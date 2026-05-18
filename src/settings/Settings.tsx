import { useEffect, useState, useRef, ReactNode } from 'react';
import StylePicker from './StylePicker';
import PermissionPanel from './PermissionPanel';
import AutoStartToggle from './AutoStartToggle';
import NotificationLog from './NotificationLog';
import MenuEditor from './menu/MenuEditor';
import HotkeyEditor from './hotkey/HotkeyEditor';
import ErrorLog from './ErrorLog';
import RadialThemeEditor from './RadialThemeEditor';
import About from './About';
import { getSettings, setSettings, DEFAULT_RADIAL_THEME, type Settings as S } from './api';

const APP_VERSION = '0.6.5';

type Tab = 'general' | 'radial' | 'diagnostics' | 'about';

const TABS: { id: Tab; label: string }[] = [
  { id: 'general', label: 'General' },
  { id: 'radial', label: 'Radial' },
  { id: 'diagnostics', label: 'Diag' },
  { id: 'about', label: 'About' },
];

interface SettingsGroupProps {
  title?: string;
  children: ReactNode;
}

const SettingsGroup = ({ title, children }: SettingsGroupProps) => (
  <div className="space-y-1.5">
    {title && (
      <h2 className="px-4 text-[13px] uppercase tracking-wider text-ios-label-secondary dark:text-ios-label-secondaryDark">
        {title}
      </h2>
    )}
    <div className="ios-card divide-y divide-ios-separator-light dark:divide-ios-separator-dark">
      {children}
    </div>
  </div>
);

export default function Settings() {
  const [state, setState] = useState<S | null>(null);
  const [tab, setTab] = useState<Tab>('general');
  const saveTimer = useRef<number | undefined>(undefined);

  useEffect(() => {
    getSettings().then(setState);
    return () => window.clearTimeout(saveTimer.current);
  }, []);

  if (!state) return <div className="p-6 text-center text-ios-label-secondary">Loading…</div>;

  const update = (next: S) => {
    setState(next);
    window.clearTimeout(saveTimer.current);
    saveTimer.current = window.setTimeout(async () => {
      await setSettings(next);
    }, 500);
  };

  return (
    <div className="min-h-screen bg-[#F2F2F7] dark:bg-black p-4 space-y-6">
      <header className="px-2 flex items-center justify-between">
        <h1 className="text-2xl font-bold tracking-tight">Settings</h1>
        <span className="text-[13px] text-ios-label-secondary dark:text-ios-label-secondaryDark bg-white/50 dark:bg-white/10 px-2 py-0.5 rounded-full">
          v{APP_VERSION}
        </span>
      </header>

      {/* iOS Segmented Control Style Tabs */}
      <nav className="p-0.5 bg-gray-200/80 dark:bg-white/10 rounded-lg flex">
        {TABS.map((t) => (
          <button
            key={t.id}
            type="button"
            onClick={() => setTab(t.id)}
            className={`flex-1 py-1.5 text-[13px] font-medium rounded-md transition-all ${
              tab === t.id
                ? 'bg-white dark:bg-white/20 shadow-sm text-ios-label-primary dark:text-white'
                : 'text-ios-label-secondary dark:text-ios-label-secondaryDark hover:text-ios-label-primary'
            }`}
          >
            {t.label}
          </button>
        ))}
      </nav>

      <main className="space-y-6">
        {tab === 'general' && (
          <>
            <PermissionPanel />
            
            <SettingsGroup title="Indicator">
              <label className="ios-item ios-item-active">
                <div className="flex flex-col">
                  <span className="ios-title">Visual Indicator</span>
                  <span className="ios-subtitle">Show at cursor on notifications</span>
                </div>
                <input
                  type="checkbox"
                  className="w-11 h-6 appearance-none bg-gray-300 dark:bg-white/10 rounded-full relative transition-colors cursor-pointer checked:bg-ios-system-green before:content-[''] before:absolute before:w-5 before:h-5 before:bg-white before:rounded-full before:top-0.5 before:left-0.5 before:transition-transform checked:before:translate-x-5"
                  checked={state.indicator_enabled ?? true}
                  onChange={(e) =>
                    update({ ...state, indicator_enabled: e.target.checked })
                  }
                />
              </label>
              <div className="p-4 bg-white/30 dark:bg-white/5">
                <StylePicker
                  value={state.indicator_style}
                  onChange={(s) => update({ ...state, indicator_style: s })}
                />
              </div>
            </SettingsGroup>

            <SettingsGroup title="System">
              <AutoStartToggle />
            </SettingsGroup>
          </>
        )}

        {tab === 'radial' && (
          <>
            <SettingsGroup title="Menu Content">
              <MenuEditor />
            </SettingsGroup>

            <SettingsGroup title="Triggers">
              <HotkeyEditor />
            </SettingsGroup>

            <SettingsGroup title="Appearance">
              <div className="p-4 bg-white/30 dark:bg-white/5">
                <RadialThemeEditor
                  value={state.radial_theme ?? DEFAULT_RADIAL_THEME}
                  onChange={(t) => update({ ...state, radial_theme: t })}
                />
              </div>
            </SettingsGroup>

            <SettingsGroup title="Behavior">
              <label className="ios-item ios-item-active">
                <div className="flex flex-col">
                  <span className="ios-title">Auto-close</span>
                  <span className="ios-subtitle">Close on cursor leave</span>
                </div>
                <input
                  type="checkbox"
                  className="w-11 h-6 appearance-none bg-gray-300 dark:bg-white/10 rounded-full relative transition-colors cursor-pointer checked:bg-ios-system-green before:content-[''] before:absolute before:w-5 before:h-5 before:bg-white before:rounded-full before:top-0.5 before:left-0.5 before:transition-transform checked:before:translate-x-5"
                  checked={state.radial_close_on_leave}
                  onChange={(e) =>
                    update({ ...state, radial_close_on_leave: e.target.checked })
                  }
                />
              </label>
            </SettingsGroup>
          </>
        )}

        {tab === 'diagnostics' && (
          <>
            <SettingsGroup title="Logs">
              <NotificationLog />
            </SettingsGroup>
            <SettingsGroup title="System Errors">
              <ErrorLog />
            </SettingsGroup>
          </>
        )}

        {tab === 'about' && (
          <div className="ios-card p-0">
            <About />
          </div>
        )}
      </main>
    </div>
  );
}
