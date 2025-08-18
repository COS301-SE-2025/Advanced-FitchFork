// vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';
import { dirname, resolve } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
    // IMPORTANT: prevent multiple "three" instances
    dedupe: ['@emotion/react', '@emotion/styled', 'three'],
  },
  // Help Vite pre-bundle the heavy deps once
  optimizeDeps: {
    include: ['three', 'three-spritetext', 'react-force-graph-3d'],
  },
  build: {
    outDir: '/var/www/fitchfork.co.za',
    emptyOutDir: true,
  },
});
