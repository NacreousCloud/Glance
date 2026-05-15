import { openUrl } from '@tauri-apps/plugin-opener';

const DEVELOPER = 'NacreousCloud';
const EMAIL = 'bebekh1216@gmail.com';
const REPO = 'https://github.com/NacreousCloud/Glance';
const ISSUES = 'https://github.com/NacreousCloud/Glance/issues';
const RELEASES = 'https://github.com/NacreousCloud/Glance/releases';
const VERSION = '0.6.4';

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
    <section className="space-y-2">
      <h2 className="text-sm font-semibold">About</h2>
      <div className="space-y-2 rounded border p-3 text-xs">
        <div className="flex items-center justify-between">
          <span className="text-gray-500">Developer</span>
          <span className="font-medium text-gray-700">{DEVELOPER}</span>
        </div>
        <div className="flex items-center justify-between">
          <span className="text-gray-500">Version</span>
          <span className="font-mono text-gray-700">v{VERSION}</span>
        </div>
        <hr className="border-gray-100" />
        <div className="space-y-1">
          <button
            type="button"
            onClick={() => open(buildReportMailto())}
            className="w-full text-left px-2 py-1 rounded border border-blue-200 bg-blue-50 hover:bg-blue-100 text-blue-700"
          >
            ✉️ Report a bug or send feedback
            <span className="block text-[10px] text-blue-500 font-mono">
              {EMAIL}
            </span>
          </button>
          <button
            type="button"
            onClick={() => open(ISSUES)}
            className="w-full text-left px-2 py-1 rounded hover:bg-gray-50 text-gray-700"
          >
            🐛 GitHub Issues
          </button>
          <button
            type="button"
            onClick={() => open(REPO)}
            className="w-full text-left px-2 py-1 rounded hover:bg-gray-50 text-gray-700"
          >
            🧭 Source code
          </button>
          <button
            type="button"
            onClick={() => open(RELEASES)}
            className="w-full text-left px-2 py-1 rounded hover:bg-gray-50 text-gray-700"
          >
            ⬇️ Releases (check for updates)
          </button>
        </div>
      </div>
    </section>
  );
}
