import Overlay from './overlay/Overlay';
import RadialMenu from './radial/RadialMenu';
import Settings from './settings/Settings';

export default function App() {
  const route = window.location.hash;
  // Visible diagnostic strip in the top-left of every window so we can tell
  // at a glance which route the webview actually loaded.
  const tag = (
    <div
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        background: 'rgba(255,0,0,0.7)',
        color: 'white',
        font: '12px monospace',
        padding: '2px 6px',
        zIndex: 99999,
      }}
    >
      hash={JSON.stringify(route)} pathname={JSON.stringify(window.location.pathname)}
    </div>
  );
  if (route.startsWith('#/overlay')) {
    return (
      <>
        {tag}
        <Overlay />
      </>
    );
  }
  if (route.startsWith('#/radial')) {
    return (
      <>
        {tag}
        <RadialMenu />
      </>
    );
  }
  return (
    <>
      {tag}
      <Settings />
    </>
  );
}
