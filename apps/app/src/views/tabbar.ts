import type { TabId } from '../app';
import { esc } from '../core/html';

const TABS: TabId[] = ['home', 'activity', 'account'];
const LABELS: Record<TabId, string> = { home: 'Home', activity: 'Activity', account: 'Account' };

function icon(name: TabId): string {
  const paths: Record<TabId, string> = {
    home: '<path d="M4 11.5 12 4l8 7.5"/><path d="M6.5 10.5V20h11v-9.5"/>',
    activity: '<path d="M6 4h12v16l-6-3.5L6 20z"/>',
    account: '<circle cx="12" cy="8" r="3.6"/><path d="M4.5 20c0-3.6 3.4-5.8 7.5-5.8s7.5 2.2 7.5 5.8"/>',
  };

  return `<svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor"
    stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">${paths[name]}</svg>`;
}

/**
 * The bar is built once and then only updated, so the glass pill can slide
 * between tabs instead of being re-created on every render.
 */
export function renderTabBar(element: HTMLElement, active: TabId, alerts: number): void {
  if (!element.firstElementChild) {
    element.innerHTML = `
      <nav class="tabbar glass" role="tablist">
        <span class="tab-pill" style="--i: 0" aria-hidden="true"></span>
        ${TABS.map(
          (tab) => `
          <button class="tab" data-tab="${tab}" role="tab" aria-selected="false">
            <span class="tab-icon">
              ${icon(tab)}
              <em class="badge" hidden></em>
            </span>
            <span>${esc(LABELS[tab])}</span>
          </button>`,
        ).join('')}
      </nav>`;
  }

  const nav = element.querySelector('.tabbar');
  if (!nav) return;

  const index = Math.max(0, TABS.indexOf(active));
  nav.querySelector<HTMLElement>('.tab-pill')?.style.setProperty('--i', String(index));

  nav.querySelectorAll<HTMLButtonElement>('[data-tab]').forEach((button) => {
    const on = button.dataset.tab === active;
    button.classList.toggle('active', on);
    button.setAttribute('aria-selected', String(on));
  });

  const badge = nav.querySelector<HTMLElement>('[data-tab="activity"] .badge');
  if (badge) {
    badge.hidden = alerts === 0;
    badge.textContent = alerts > 9 ? '9+' : String(alerts);
  }
}
