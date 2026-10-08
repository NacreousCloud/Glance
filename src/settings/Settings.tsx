import { useState, ReactNode } from 'react';
import StylePicker from './StylePicker';
import PermissionPanel from './PermissionPanel';
import AutoStartToggle from './AutoStartToggle';
import NotificationLog from './NotificationLog';
import MenuEditor from './menu/MenuEditor';
import HotkeyEditor from './hotkey/HotkeyEditor';
import ErrorLog from './ErrorLog';
import RadialThemeEditor from './RadialThemeEditor';
import About from './About';
import { DEFAULT_RADIAL_THEME } from './api';
import { usePreferences } from './usePreferences';
import GettingStarted from './GettingStarted';

const APP_VERSION = '0.6.7';

type Tab = 'general' | 'radial' | 'diagnostics' | 'about';

const TABS: { id: Tab; label: string }[] = [
  { id: 'general', label: 'General' },
  { id: 'radial', label: 'Radial Menu' },
  { id: 'diagnostics', label: 'Diagnostics' },
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
    <div className="divide-y ios-card divide-ios-separator-light dark:divide-ios-separator-dark">
      {children}
    </div>
  </div>
);

export default function Settings() {
  const { state, update, error, saving, retry } = usePreferences();
  const [tab, setTab] = useState<Tab>('general');
  if (!state) return <div className="p-6 text-center text-ios-label-secondary">
    {error ? <><p role="alert">{error}</p><button onClick={retry}>Retry</button></> : 'Loading…'}
  </div>;

  return (
    <div className="min-h-screen bg-[#F2F2F7] dark:bg-black p-4 space-y-6">
      <header className="flex items-center justify-between px-2">
        <h1 className="text-2xl font-bold tracking-tight">Settings</h1>
        <span className="text-[13px] text-ios-label-secondary dark:text-ios-label-secondaryDark bg-white/50 dark:bg-white/10 px-2 py-0.5 rounded-full">
          v{APP_VERSION}
        </span>
      </header>

      {saving && <p role="status" className="px-2 text-sm">Saving…</p>}
      {error && <div role="alert" className="p-3 ios-card text-ios-system-red">
        {error} <button type="button" className="underline" onClick={retry}>Retry save</button>
      </div>}

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
            <GettingStarted completed={!!state.onboarding_completed}
              theme={state.radial_theme ?? DEFAULT_RADIAL_THEME} saving={saving}
              onComplete={() => update({ onboarding_completed: true })}
              onConfigure={() => setTab('radial')} />
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
                    update({ indicator_enabled: e.target.checked })
                  }
                />
              </label>
              <div className="p-4 bg-white/30 dark:bg-white/5">
                <StylePicker
                  value={state.indicator_style}
                  onChange={(s) => update({ indicator_style: s })}
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
                  onChange={(t) => update({ radial_theme: t })}
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
                    update({ radial_close_on_leave: e.target.checked })
                  }
                />
              </label>
            </SettingsGroup>
          </>
        )}

        {tab === 'diagnostics' && (
          <>
            <div className="space-y-1.5">
              <h2 className="px-4 text-[13px] uppercase tracking-wider text-ios-label-secondary dark:text-ios-label-secondaryDark">
                Notification Logs
              </h2>
              <NotificationLog />
            </div>
            <ErrorLog />
          </>
        )}

        {tab === 'about' && (
          <div className="p-0 ios-card">
            <About />
          </div>
        )}
      </main>
    </div>
  );
}
