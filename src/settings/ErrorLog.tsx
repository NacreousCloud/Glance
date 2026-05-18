import { useEffect, useState } from 'react';
import { getRecentErrors, clearErrors, type ErrorEntry } from './api';

export default function ErrorLog() {
  const [errors, setErrors] = useState<ErrorEntry[]>([]);
  const [expanded, setExpanded] = useState<string | null>(null);

  useEffect(() => {
    const refresh = () => getRecentErrors().then(setErrors).catch(() => {});
    refresh();
    const id = window.setInterval(refresh, 2000);
    return () => window.clearInterval(id);
  }, []);

  return (
    <div className="space-y-1.5">
      <div className="flex items-center justify-between px-4">
        <h2 className="text-[13px] uppercase tracking-wider text-ios-label-secondary dark:text-ios-label-secondaryDark">
          System Errors {errors.length > 0 && `(${errors.length})`}
        </h2>
        {errors.length > 0 && (
          <button
            type="button"
            className="text-[13px] font-medium text-ios-system-red active:opacity-60"
            onClick={() => {
              clearErrors().then(() => setErrors([]));
            }}
          >
            Clear All
          </button>
        )}
      </div>

      <div className="ios-card divide-y divide-ios-separator-light dark:divide-ios-separator-dark">
        {errors.length === 0 ? (
          <div className="p-8 text-[15px] text-ios-label-secondary dark:text-ios-label-secondaryDark text-center italic">
            No errors recorded.
          </div>
        ) : (
          errors.map((e) => {
            const isOpen = expanded === e.id;
            const firstLine = e.message.split('\n')[0];
            return (
              <div key={e.id} className="p-4 space-y-1 bg-ios-system-red/[0.02] active:bg-ios-system-red/[0.05] transition-colors">
                <div className="flex justify-between items-baseline">
                  <span className="text-[11px] font-bold text-ios-system-red uppercase tracking-tight">
                    {e.item_label || 'System'}
                  </span>
                  <span className="text-[11px] text-ios-label-secondary dark:text-ios-label-secondaryDark">
                    {new Date(e.timestamp_ms).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })}
                  </span>
                </div>
                <div className="text-[13px] font-mono text-ios-system-red break-all leading-tight">
                  {isOpen ? e.message : firstLine}
                </div>
                {(e.message.includes('\n') || e.message.length > 60) && (
                  <button
                    type="button"
                    className="text-[11px] font-semibold text-ios-system-blue uppercase mt-1"
                    onClick={() => setExpanded(isOpen ? null : e.id)}
                  >
                    {isOpen ? 'Show Less' : 'Read More'}
                  </button>
                )}
              </div>
            );
          })
        )}
      </div>
    </div>
  );
}
