import { useEffect, useState } from 'react';
import { getRecentErrors, clearErrors, type ErrorEntry } from './api';

function formatTime(ms: number) {
  const d = new Date(ms);
  const hh = String(d.getHours()).padStart(2, '0');
  const mm = String(d.getMinutes()).padStart(2, '0');
  const ss = String(d.getSeconds()).padStart(2, '0');
  return `${hh}:${mm}:${ss}`;
}

export default function ErrorLog() {
  const [errors, setErrors] = useState<ErrorEntry[]>([]);
  const [expanded, setExpanded] = useState<string | null>(null);

  useEffect(() => {
    const refresh = () => getRecentErrors().then(setErrors).catch(() => {});
    refresh();
    const id = window.setInterval(refresh, 2000);
    return () => window.clearInterval(id);
  }, []);

  if (errors.length === 0) {
    return (
      <section className="space-y-2">
        <h2 className="text-sm font-semibold">Recent errors</h2>
        <p className="text-xs text-gray-400">No errors recorded.</p>
      </section>
    );
  }

  return (
    <section className="space-y-2">
      <div className="flex items-center justify-between">
        <h2 className="text-sm font-semibold">
          Recent errors{' '}
          <span className="text-xs text-gray-400 font-normal">
            ({errors.length})
          </span>
        </h2>
        <button
          type="button"
          className="text-xs text-gray-500 hover:text-red-600"
          onClick={() => {
            clearErrors().then(() => setErrors([]));
          }}
        >
          Clear
        </button>
      </div>
      <ul className="space-y-1 max-h-64 overflow-y-auto">
        {errors.map((e) => {
          const isOpen = expanded === e.id;
          const firstLine = e.message.split('\n')[0];
          return (
            <li
              key={e.id}
              className="border border-red-200 bg-red-50 rounded p-2 text-xs"
            >
              <div className="flex items-start justify-between gap-2">
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2 text-gray-500">
                    <span className="font-mono">{formatTime(e.timestamp_ms)}</span>
                    <span className="font-medium text-gray-700 truncate">
                      {e.item_label}
                    </span>
                  </div>
                  <div className="font-mono text-red-700 break-all">
                    {isOpen ? e.message : firstLine}
                  </div>
                </div>
                {e.message.includes('\n') || e.message.length > 80 ? (
                  <button
                    type="button"
                    className="text-gray-500 hover:text-blue-600 shrink-0"
                    onClick={() => setExpanded(isOpen ? null : e.id)}
                  >
                    {isOpen ? 'Collapse' : 'More'}
                  </button>
                ) : null}
              </div>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
