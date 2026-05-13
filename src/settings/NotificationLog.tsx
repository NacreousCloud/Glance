import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { getRecentEvents, type NotiEvent } from './api';

export default function NotificationLog() {
  const [logs, setLogs] = useState<NotiEvent[]>([]);

  useEffect(() => {
    // Load initial history
    getRecentEvents().then((events) => {
      setLogs(events.reverse()); // Show newest first
    });

    // Listen for live updates
    const un = listen<any>('noti:show', (e) => {
      // The event from noti:show matches our NotiEvent structure (with id, app_name, etc)
      setLogs((prev) => [e.payload, ...prev].slice(0, 50)); // Keep last 50
    });

    return () => {
      un.then((f) => f());
    };
  }, []);

  return (
    <div className="space-y-3">
      <h2 className="text-sm font-semibold text-gray-500 uppercase tracking-wider">
        Debug Log (Last 50)
      </h2>
      <div className="bg-gray-50 rounded-lg border border-gray-100 overflow-hidden">
        {logs.length === 0 ? (
          <div className="p-4 text-sm text-gray-400 text-center italic">
            No notifications recorded yet
          </div>
        ) : (
          <div className="divide-y divide-gray-100 max-h-64 overflow-y-auto">
            {logs.map((log) => (
              <div key={log.id} className="p-3 text-xs space-y-1 hover:bg-white transition-colors">
                <div className="flex justify-between items-start">
                  <span className="font-bold text-gray-700">{log.app_name}</span>
                  <span className="text-[10px] text-gray-400 font-mono">
                    {new Date(Number(log.id / 1000000n)).toLocaleTimeString()}
                  </span>
                </div>
                <div className="font-medium text-gray-600 truncate">{log.title}</div>
                {log.body && <div className="text-gray-400 line-clamp-1">{log.body}</div>}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
