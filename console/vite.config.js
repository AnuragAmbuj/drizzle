import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:3000',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
      '/observability/metrics': {
        target: 'http://127.0.0.1:9111',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/observability\/metrics/, '/metrics'),
      },
      '/observability': {
        target: 'http://127.0.0.1:9111',
        changeOrigin: true,
      }
    }
  }
})
