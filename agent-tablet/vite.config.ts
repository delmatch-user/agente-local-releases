import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { VitePWA } from 'vite-plugin-pwa'

export default defineConfig({
  plugins: [
    react(),
    VitePWA({ 
      registerType: 'autoUpdate',
      injectRegister: 'inline',
      workbox: {
        globPatterns: ['**/*.{js,css,html,ico,png,svg}'],
        cleanupOutdatedCaches: true,
        clientsClaim: true,
        skipWaiting: true
      },
      devOptions: {
        enabled: true
      },
      manifest: {
        name: 'MiaCardapio Agent Tablet',
        short_name: 'MiaAgent',
        description: 'Módulo de Gestão de Impressão',
        theme_color: '#ffffff',
        background_color: '#f8fafc',
        display: 'standalone',
        icons: [
          {
            src: 'https://cdn-icons-png.flaticon.com/512/877/877085.png', // Icone de tablet/cardápio genérico
            sizes: '512x512',
            type: 'image/png'
          }
        ]
      }
    })
  ]
})
