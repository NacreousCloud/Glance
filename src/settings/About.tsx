import { openUrl } from '@tauri-apps/plugin-opener';

const DEVELOPER = 'NacreousCloud';
const EMAIL = 'bebekh1216@gmail.com';
const REPO = 'https://github.com/NacreousCloud/Glance';
const ISSUES = 'https://github.com/NacreousCloud/Glance/issues';
const RELEASES = 'https://github.com/NacreousCloud/Glance/releases';
const VERSION = '0.6.5';

function buildReportMailto(): string {
  const subject = encodeURIComponent(`[Glance ${VERSION}] Bug report / Feedback`);
  const body = encodeURIComponent(
    [
      'What happened:',
      '',
      '',
      'Steps to reproduce:',
      '1. ',
      '2. ',
      '3. ',
      '',
      'Expected vs. actual:',
      '',
      '',
      '---',
      `App version: ${VERSION}`,
      `OS: ${navigator.platform}`,
      `User agent: ${navigator.userAgent}`,
    ].join('\n')
  );
  return `mailto:${EMAIL}?subject=${subject}&body=${body}`;
}

export default function About() {
  const open = (url: string) => {
    openUrl(url).catch(() => {});
  };

  return (
    <div className="space-y-6">
      <div className="flex flex-col items-center py-6">
        <div className="w-20 h-20 rounded-[22%] bg-gradient-to-br from-ios-system-blue to-blue-600 shadow-xl flex items-center justify-center text-white text-4xl font-bold mb-3 overflow-hidden">
          G
        </div>
        <h2 className="text-xl font-bold tracking-tight">Glance</h2>
        <p className="text-[13px] text-ios-label-secondary dark:text-ios-label-secondaryDark mt-1">
          Version {VERSION} ({DEVELOPER})
        </p>
      </div>

      <div className="ios-card divide-y divide-ios-separator-light dark:divide-ios-separator-dark">
        <button
          type="button"
          onClick={() => open(buildReportMailto())}
          className="ios-item ios-item-active w-full text-left"
        >
          <span className="ios-title text-ios-system-blue">Send Feedback</span>
          <span className="text-ios-system-blue opacity-50">→</span>
        </button>
        <button
          type="button"
          onClick={() => open(ISSUES)}
          className="ios-item ios-item-active w-full text-left"
        >
          <span className="ios-title">GitHub Issues</span>
          <span className="text-ios-label-secondary">→</span>
        </button>
        <button
          type="button"
          onClick={() => open(REPO)}
          className="ios-item ios-item-active w-full text-left"
        >
          <span className="ios-title">Source Code</span>
          <span className="text-ios-label-secondary">→</span>
        </button>
        <button
          type="button"
          onClick={() => open(RELEASES)}
          className="ios-item ios-item-active w-full text-left"
        >
          <span className="ios-title">Release Notes</span>
          <span className="text-ios-label-secondary">→</span>
        </button>
      </div>

      <p className="px-4 text-[11px] text-center text-ios-label-secondary dark:text-ios-label-secondaryDark uppercase tracking-widest">
        © 2026 {DEVELOPER}
      </p>
    </div>
  );
}
