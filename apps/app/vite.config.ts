import { defineConfig } from 'vite';

// Tauri needs a fixed dev server and no screen clearing.
const host = process.env.TAURI_DEV_HOST;
const rpcTarget = process.env.VITE_RPC_URL ?? 'http://127.0.0.1:50051';

export default defineConfig({
  clearScreen: false,
  server: {
    // 5173 is often taken/reserved on Windows, so this project uses 4180.
    port: 4180,
    strictPort: true,
    host: host || '127.0.0.1',
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
    watch: { ignored: ['**/src-tauri/**'] },
    proxy: {
      // gRPC-Web paths are /smarttrash.<package>.<Service>/<Method>.
      // With this proxy the browser talks to its own origin: no CORS, and the
      // backend address lives only in this file.
      '/smarttrash': {
        target: rpcTarget,
        changeOrigin: false,
      },
    },
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: 'es2021',
    minify: 'esbuild',
    sourcemap: false,
    outDir: 'dist',
    emptyOutDir: true,
  },
});
