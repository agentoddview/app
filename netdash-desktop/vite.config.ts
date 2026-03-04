import { defineConfig } from 'vite';

export default defineConfig({
  clearScreen: false,
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    rollupOptions: {
      input: {
        index: 'index.html',
        settings: 'settings.html'
      }
    }
  }
});
