import './style.css';
import { mountMap } from './core/map';
import { boot } from './app';

mountMap(document.getElementById('map') as HTMLElement);

const splash = document.getElementById('splash');
const started = Date.now();

boot();

// Keep the splash for a beat so the first paint is never a blank frame.
const remaining = Math.max(0, 800 - (Date.now() - started));
window.setTimeout(() => {
  splash?.classList.add('gone');
  window.setTimeout(() => splash?.remove(), 420);
}, remaining);
