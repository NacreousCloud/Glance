import Overlay from './overlay/Overlay';
import RadialMenu from './radial/RadialMenu';
import Settings from './settings/Settings';

export default function App() {
  const route = window.location.hash;
  if (route.startsWith('#/overlay')) return <Overlay />;
  if (route.startsWith('#/radial')) return <RadialMenu />;
  return <Settings />;
}
