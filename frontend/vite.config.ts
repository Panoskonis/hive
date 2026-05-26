import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

const apiProxyTarget = process.env.VITE_PROXY_TARGET ?? 'http://127.0.0.1:3000'

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte()],
  server: {
    proxy: {
      '/api': {
        target: apiProxyTarget,
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
    },
  },
})
