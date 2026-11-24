import type { Actions, AppState } from '../app';
import { rpcUrl } from '../core/config';
import { esc } from '../core/html';
import { platform } from '../core/platform';
import { Role } from '../generated/identity';

const USERS = [
  { id: 'HOUSEHOLD-0421', role: 'household', hint: 'Request a collection' },
  { id: 'BUSINESS-12', role: 'business', hint: 'Recurring pickups' },
  { id: 'DRIVER-07', role: 'driver', hint: "Today's route" },
  { id: 'COUNCIL-01', role: 'council', hint: 'All pickups' },
];

const ROLES = [
  { value: Role.HOUSEHOLD, label: 'household' },
  { value: Role.BUSINESS, label: 'business' },
  { value: Role.DRIVER, label: 'driver' },
  { value: Role.COUNCIL, label: 'council' },
];

function signInCard(state: AppState): string {
  return `
    <section class="card glass">
      <h2>Sign in</h2>
      <div class="user-list">
        ${USERS.map(
          (user) => `
          <button class="user-button" data-user="${esc(user.id)}" ${state.busy ? 'disabled' : ''}>
            <span>
              <span class="role">${esc(user.role)}</span>
              <span class="id">${esc(user.id)}</span>
            </span>
            <span class="hint">${esc(user.hint)}</span>
          </button>`,
        ).join('')}
      </div>
    </section>`;
}

function cameraBlock(state: AppState): string {
  return `
    <div class="avatar-row">
      <div class="avatar ${state.avatar ? 'has-photo' : ''}">
        ${state.avatar ? `<img src="${esc(state.avatar)}" alt="profile" />` : '<span>ST</span>'}
      </div>
      <div class="avatar-actions">
        <button type="button" class="ghost small" id="open-camera">
          ${state.avatar ? 'Retake photo' : 'Add photo'}
        </button>
        ${state.cameraOn ? '<button type="button" class="ghost small" id="stop-camera">Stop</button>' : ''}
      </div>
    </div>
    ${
      state.cameraOn
        ? `<div class="camera">
             <video id="cam" playsinline muted></video>
             <button type="button" class="block" id="capture">Capture</button>
           </div>`
        : ''
    }`;
}

function registerCard(state: AppState): string {
  return `
    <section class="card glass">
      <h2>Create account</h2>
      <form id="register-form" class="form">
        ${cameraBlock(state)}

        <label class="field">
          <span>Name</span>
          <input name="display_name" required maxlength="80" placeholder="e.g. Asha Mwinyi" />
        </label>

        <div class="row-2">
          <label class="field">
            <span>I am a</span>
            <select name="role">
              ${ROLES.map((r) => `<option value="${r.value}">${esc(r.label)}</option>`).join('')}
            </select>
          </label>
          <label class="field">
            <span>Ward</span>
            <input name="ward" required maxlength="64" placeholder="Kinondoni" />
          </label>
        </div>

        <button type="button" class="ghost small" id="locate" ${state.geoBusy ? 'disabled' : ''}>
          ${state.geoBusy ? 'Locating…' : 'Use my current location'}
        </button>

        <div class="row-2">
          <label class="field">
            <span>Latitude</span>
            <input name="latitude" type="number" step="any" value="-6.7924" required />
          </label>
          <label class="field">
            <span>Longitude</span>
            <input name="longitude" type="number" step="any" value="39.2083" required />
          </label>
        </div>

        <button type="submit" ${state.busy ? 'disabled' : ''}>Create account</button>
      </form>
    </section>`;
}

export function loginView(state: AppState): string {
  const register = state.authMode === 'register';

  return `
    <div class="login">
      <div class="brand">
        <span class="mark">ST</span>
        <span class="brand-text">
          <strong>Smart Trash</strong>
          <small>Collection requests, one tap</small>
        </span>
      </div>

      ${state.error ? `<div class="notice">${esc(state.error)}</div>` : ''}
      ${state.notice ? `<div class="notice ok">${esc(state.notice)}</div>` : ''}

      <div class="switcher">
        <button class="switch ${register ? '' : 'active'}" data-mode="signin">Sign in</button>
        <button class="switch ${register ? 'active' : ''}" data-mode="register">Create account</button>
      </div>

      ${register ? registerCard(state) : signInCard(state)}

      <details class="endpoint">
        <summary>Server · ${esc(rpcUrl())}</summary>
        <label class="field">
          <span>gRPC address</span>
          <input id="base" value="${esc(rpcUrl())}" autocomplete="off" spellcheck="false" />
        </label>
        <button class="ghost small" id="save-base">Save address</button>
      </details>

      <p class="foot">gRPC-Web · AuthService · ${esc(platform)}</p>
    </div>`;
}

export function bindLogin(root: HTMLElement, actions: Actions): void {
  root.querySelectorAll<HTMLButtonElement>('[data-mode]').forEach((button) => {
    button.addEventListener('click', () =>
      actions.setAuthMode(button.dataset.mode === 'register' ? 'register' : 'signin'),
    );
  });

  root.querySelectorAll<HTMLButtonElement>('[data-user]').forEach((button) => {
    button.addEventListener('click', () => actions.signIn(button.dataset.user ?? ''));
  });

  root.querySelector<HTMLButtonElement>('#save-base')?.addEventListener('click', () => {
    const input = root.querySelector<HTMLInputElement>('#base');
    if (input) actions.saveServer(input.value);
  });

  root.querySelector<HTMLButtonElement>('#locate')?.addEventListener('click', () => actions.locate());

  root.querySelector<HTMLButtonElement>('#open-camera')?.addEventListener('click', () =>
    actions.startCamera(),
  );
  root.querySelector<HTMLButtonElement>('#stop-camera')?.addEventListener('click', () =>
    actions.stopCamera(),
  );
  root.querySelector<HTMLButtonElement>('#capture')?.addEventListener('click', () =>
    actions.capturePhoto(),
  );

  const video = root.querySelector<HTMLVideoElement>('#cam');
  if (video) actions.attachCamera(video);

  root.querySelector<HTMLFormElement>('#register-form')?.addEventListener('submit', (event) => {
    event.preventDefault();
    actions.register(event.currentTarget as HTMLFormElement);
  });
}
