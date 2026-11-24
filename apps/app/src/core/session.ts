// Login / logout. The token never leaves this module except as a header.

import { AuthServiceClient } from '../generated/auth.client';
import type { RegisterRequest } from '../generated/auth';
import type { Role } from '../generated/identity';
import { getTransport, setAuthToken } from './transport';

export interface Session {
  userId: string;
  role: Role;
  token: string;
  expiresAtUnix: string;
}

const KEY = 'smarttrash.session';

let current: Session | null = null;

export function session(): Session | null {
  return current;
}

export function isExpired(value: Session): boolean {
  const seconds = Number(value.expiresAtUnix);
  return Number.isFinite(seconds) && seconds * 1000 <= Date.now();
}

export async function login(userId: string): Promise<Session> {
  const client = new AuthServiceClient(getTransport());
  const { response } = await client.login({ userId });
  return store(response);
}

/** Create an account and stay signed in. */
export async function register(request: RegisterRequest): Promise<Session> {
  const client = new AuthServiceClient(getTransport());
  const { response } = await client.register(request);
  return store(response);
}

function store(response: { userId: string; role: Role; token: string; expiresAtUnix: string }): Session {
  const next: Session = {
    userId: response.userId,
    role: response.role,
    token: response.token,
    expiresAtUnix: response.expiresAtUnix,
  };

  current = next;
  setAuthToken(next.token);
  localStorage.setItem(KEY, JSON.stringify(next));
  return next;
}

/** Bring back a stored session, if it is still valid. */
export function restore(): Session | null {
  const raw = localStorage.getItem(KEY);
  if (!raw) return null;

  try {
    const parsed = JSON.parse(raw) as Session;
    if (!parsed.token || isExpired(parsed)) {
      logout();
      return null;
    }
    current = parsed;
    setAuthToken(parsed.token);
    return parsed;
  } catch {
    logout();
    return null;
  }
}

export function logout(): void {
  current = null;
  setAuthToken('');
  localStorage.removeItem(KEY);
}
