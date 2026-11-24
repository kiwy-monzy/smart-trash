// Where the gRPC endpoint lives (production builds only — the dev server
// proxies /smarttrash… to it, see vite.config.ts).

import { isAndroid } from './platform';

const KEY = 'smarttrash.rpc';

export const DEFAULT_RPC = isAndroid ? 'http://10.0.2.2:50051' : 'http://127.0.0.1:50051';

export function rpcUrl(): string {
  return localStorage.getItem(KEY) || DEFAULT_RPC;
}

export function saveRpcUrl(url: string): void {
  const clean = url.trim().replace(/\/+$/, '');
  localStorage.setItem(KEY, clean || DEFAULT_RPC);
}
