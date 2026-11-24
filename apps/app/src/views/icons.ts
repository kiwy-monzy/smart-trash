// Inline SVG icons for the tab bar (no icon font, no extra requests).

const wrap = (path: string) =>
  `<svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${path}</svg>`;

export const icons = {
  pickups: wrap('<path d="M4 7h16M4 12h16M4 17h10"/>'),
  route: wrap('<path d="M6 19a3 3 0 1 0 0-6 3 3 0 0 0 0 6ZM18 11a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z"/><path d="M9 16h4a3 3 0 0 0 3-3V9"/>'),
  request: wrap('<path d="M12 5v14M5 12h14"/>'),
  account: wrap('<circle cx="12" cy="8" r="3.4"/><path d="M5 20c0-3.3 3.1-5.4 7-5.4s7 2.1 7 5.4"/>'),
  truck: wrap('<path d="M3 7h11v8H3zM14 10h4l3 3v2h-7z"/><circle cx="7" cy="17.5" r="1.6"/><circle cx="17.5" cy="17.5" r="1.6"/>'),
  calendar: wrap('<rect x="3.5" y="5" width="17" height="15" rx="2.5"/><path d="M3.5 10h17M8 3v4M16 3v4"/>'),
  bell: wrap('<path d="M18 15H6l1.5-2v-4a4.5 4.5 0 0 1 9 0v4L18 15Z"/><path d="M10 18a2 2 0 0 0 4 0"/>'),
};
