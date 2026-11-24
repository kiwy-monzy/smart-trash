import type { Actions, AppState } from '../app';
import { rpcUrl } from '../core/config';
import { roleLabel } from '../core/format';
import { esc } from '../core/html';
import { platform } from '../core/platform';

export function accountView(state: AppState): string {
  const user = state.session!;
  const end = Number(user.expiresAtUnix) * 1000;

  return `
    <div class="page plain">
      <header class="page-head"><h1>Account</h1></header>

      <section class="profile">
        <div class="avatar">${state.avatar ? `<img src="${esc(state.avatar)}" alt="" />` : 'ST'}</div>
        <div class="profile-text">
          <strong>${esc(user.userId)}</strong>
          <span>${esc(roleLabel(user.role))} · ${esc(platform)}</span>
        </div>
        <span class="rating">★ 4.88</span>
      </section>

      <div class="promo">
        <span class="promo-icon">🛡️</span>
        <div>
          <strong>Signed in securely</strong>
          <span>Session ends ${esc(Number.isFinite(end) ? new Date(end).toLocaleString() : '—')}</span>
        </div>
      </div>

      <ul class="menu">
        <li>
          <button data-account="endpoint">
            <span class="menu-icon">🌐</span>
            <span>Endpoint</span>
            <span class="menu-value">${esc(rpcUrl())}</span>
            <span class="chev">›</span>
          </button>
        </li>
        <li>
          <button data-tab="activity" data-activity="alerts">
            <span class="menu-icon">🔔</span>
            <span>Alerts</span>
            <span class="menu-value">${state.notifications.length}</span>
            <span class="chev">›</span>
          </button>
        </li>
        <li>
          <button data-account="profile">
            <span class="menu-icon">👤</span>
            <span>Profile</span>
            <span class="menu-value">${esc(user.userId)}</span>
            <span class="chev">›</span>
          </button>
        </li>
      </ul>

      ${
        state.accountPanel === 'endpoint'
          ? `<div class="panel">
               <label class="field"><span>gRPC address</span>
                 <input id="base" value="${esc(state.rpcUrl)}" autocomplete="off" spellcheck="false" />
               </label>
               <button class="ghost block" id="save-base">Save address</button>
             </div>`
          : ''
      }

      ${
        state.accountPanel === 'profile'
          ? `<div class="panel">
               <dl class="kv">
                 <div><dt>User</dt><dd>${esc(user.userId)}</dd></div>
                 <div><dt>Role</dt><dd>${esc(roleLabel(user.role))}</dd></div>
                 <div><dt>Platform</dt><dd>${esc(platform)}</dd></div>
                 <div><dt>Session ends</dt><dd>${esc(Number.isFinite(end) ? new Date(end).toLocaleString() : '—')}</dd></div>
               </dl>
             </div>`
          : ''
      }

      <button class="danger block" id="signout">Sign out</button>
      <p class="foot">Smart Trash · ${esc(platform)}<br />Map data © OpenStreetMap contributors (ODbL)<br />Wards: NBS 2022 census</p>
    </div>`;
}

export function bindAccount(root: HTMLElement, actions: Actions): void {
  root.querySelectorAll<HTMLButtonElement>('[data-account]').forEach((button) => {
    button.addEventListener('click', () => actions.togglePanel(button.dataset.account as never));
  });

  root.querySelector<HTMLButtonElement>('#save-base')?.addEventListener('click', () => {
    const input = root.querySelector<HTMLInputElement>('#base');
    if (input) actions.saveServer(input.value);
  });

  root.querySelector<HTMLButtonElement>('#signout')?.addEventListener('click', () => actions.signOut());
}
