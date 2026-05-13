import Overlay from './overlay/Overlay';

export default function App() {
  const route = window.location.hash;
  if (route.startsWith('#/overlay')) return <Overlay />;
  return <div style={{ padding: 24 }}>Settings (TBD — Task 15)</div>;
}
