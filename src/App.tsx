import Overlay from './overlay/Overlay';
import Settings from './settings/Settings';

export default function App() {
  const route = window.location.hash;
  if (route.startsWith('#/overlay')) return <Overlay />;
  return <Settings />;
}
