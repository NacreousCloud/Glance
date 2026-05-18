import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { getRecentEvents, type NotiEvent } from './api';

export default function NotificationLog() {
  const [logs, setLogs] = useState<NotiEvent[]>([]);

  useEffect(() => {
    // Load initial history
    getRecentEvents().then((events) => {
      setLogs([...events].reverse()); // Show newest first
    });

    // Listen for live updates
    const un = listen<NotiEvent>('noti:show', (e) => {
      setLogs((prev) => [e.payload, ...prev].slice(0, 50));
    });

    return () => {
      un.then((f) => f());
    };
  }, []);

  return (
    <div className="ios-card max-h-[300px] overflow-y-auto divide-y divide-ios-separator-light dark:divide-ios-separator-dark">
      {logs.length === 0 ? (
        <div className="p-8 text-[15px] text-ios-label-secondary dark:text-ios-label-secondaryDark text-center italic">
          No notifications recorded yet
        </div>
      ) : (
        logs.map((log) => (
          <div key={log.id} className="p-4 space-y-0.5 hover:bg-black/[0.02] dark:hover:bg-white/[0.02] transition-colors">
            <div className="flex justify-between items-baseline">
              <span className="text-[13px] font-bold text-ios-system-blue uppercase tracking-tight">
                {log.app_name}
              </span>
              <span className="text-[11px] text-ios-label-secondary dark:text-ios-label-secondaryDark">
                {new Date(log.timestamp_ms).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
              </span>
            </div>
            <div className="ios-title leading-tight">{log.title}</div>
            {log.body && (
              <div className="text-[13px] text-ios-label-secondary dark:text-ios-label-secondaryDark line-clamp-2 leading-snug">
                {log.body}
              </div>
            )}
          </div>
        ))
      )}
    </div>
  );
}
