import type { Actions, AppState } from '../app';
import { coordinate, statusClass, statusLabel, volume, wasteLabel } from '../core/format';
import { esc } from '../core/html';

export function activityView(state: AppState): string {
  const pickups = `
    ${
      state.pickups.length
        ? `<div class="rows">${state.pickups
            .map(
              (p) => `
        <button class="row" data-pickup="${esc(p.pickupId)}">
          <span class="row-icon">🗑️</span>
          <span class="row-main">
            <strong>${esc(p.pickupId)}</strong>
            <small>${esc(wasteLabel(p.wasteType))} · ${esc(volume(p.estimatedVolume))}</small>
            <small class="mono">${esc(coordinate(p.latitude))}, ${esc(coordinate(p.longitude))}</small>
          </span>
          <span class="pill ${statusClass(p.status)}">${esc(statusLabel(p.status))}</span>
        </button>`,
            )
            .join('')}</div>`
        : '<p class="empty">No pickups yet.</p>'
    }`;

  const alerts = `
    ${
      state.notifications.length
        ? `<div class="rows">${state.notifications
            .map(
              (n) => `
        <div class="row static">
          <span class="row-icon">🔔</span>
          <span class="row-main">
            <strong>${esc(n.title)}</strong>
            <small>${esc(n.body)}</small>
            <small class="mono">${esc(n.h3Cell)}</small>
          </span>
        </div>`,
            )
            .join('')}</div>`
        : '<p class="empty">No alerts yet.</p>'
    }`;

  return `
    <div class="page plain">
      <header class="page-head"><h1>Activity</h1></header>

      <div class="segmented pad">
        <button class="seg ${state.activity === 'pickups' ? 'active' : ''}" data-activity="pickups">
          Pickups <em>${state.pickups.length}</em>
        </button>
        <button class="seg ${state.activity === 'alerts' ? 'active' : ''}" data-activity="alerts">
          Alerts <em>${state.notifications.length}</em>
        </button>
      </div>

      <div class="page-body">${state.activity === 'alerts' ? alerts : pickups}</div>
    </div>`;
}

export function bindActivity(root: HTMLElement, actions: Actions): void {
  root.querySelectorAll<HTMLButtonElement>('[data-activity]').forEach((button) => {
    button.addEventListener('click', () =>
      actions.setActivity(button.dataset.activity === 'alerts' ? 'alerts' : 'pickups'),
    );
  });

  root.querySelectorAll<HTMLButtonElement>('[data-pickup]').forEach((button) => {
    button.addEventListener('click', () => actions.focusPickup(button.dataset.pickup ?? ''));
  });
}
