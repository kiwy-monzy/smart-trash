// gRPC-Web transport shared by every client.
// The token is added here, once, instead of at each call site.

import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';
import type { RpcInterceptor } from '@protobuf-ts/runtime-rpc';

import { rpcUrl } from './config';

let token = '';
let transport: GrpcWebFetchTransport | undefined;

const auth: RpcInterceptor = {
  interceptUnary(next, method, input, options) {
    if (token) options.meta = { ...options.meta, authorization: `Bearer ${token}` };
    return next(method, input, options);
  },
};

function create(): GrpcWebFetchTransport {
  return new GrpcWebFetchTransport({
    // In dev the Vite proxy forwards /smarttrash… to the RPC server, so the
    // browser talks to its own origin and the address lives in vite.config.ts.
    baseUrl: import.meta.env.DEV ? '' : rpcUrl(),
    format: 'binary',
    interceptors: [auth],
  });
}

export function getTransport(): GrpcWebFetchTransport {
  transport ??= create();
  return transport;
}

export function setAuthToken(value: string): void {
  token = value;
}

/** Call after the server address changes. */
export function resetTransport(): void {
  transport = undefined;
}
