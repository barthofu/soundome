/// <reference lib="webworker" />

import { cleanupOutdatedCaches, precacheAndRoute } from 'workbox-precaching'
import { registerRoute } from 'workbox-routing'
import { NetworkFirst, CacheFirst } from 'workbox-strategies'
import { CacheExpiration } from 'workbox-expiration'

declare const self: ServiceWorkerGlobalScope

// Cleanup old caches
cleanupOutdatedCaches()

// Precache and route production assets
precacheAndRoute(self.__WB_MANIFEST)

// Handle API requests with Network First strategy
registerRoute(
  ({ url }) => url.pathname.startsWith('/api'),
  new NetworkFirst({
    cacheName: 'api-cache',
    plugins: [
      new CacheExpiration({
        maxEntries: 50,
        maxAgeSeconds: 300, // 5 minutes
      }),
    ],
  })
)

// Handle local API requests in development
registerRoute(
  ({ url }) => url.origin === 'http://localhost:8000' && url.pathname.startsWith('/api'),
  new NetworkFirst({
    cacheName: 'local-api-cache',
    plugins: [
      new CacheExpiration({
        maxEntries: 50,
        maxAgeSeconds: 300, // 5 minutes
      }),
    ],
  })
)

// Handle assets with Cache First strategy
registerRoute(
  ({ request }) => request.destination === 'style' || request.destination === 'script' || request.destination === 'image',
  new CacheFirst({
    cacheName: 'assets-cache',
    plugins: [
      new CacheExpiration({
        maxEntries: 100,
        maxAgeSeconds: 60 * 60 * 24 * 7, // 1 week
      }),
    ],
  })
)

// Handle navigation requests
registerRoute(
  ({ request }) => request.mode === 'navigate',
  new NetworkFirst({
    cacheName: 'pages-cache',
  })
)

// Listen for messages from the app
self.addEventListener('message', (event) => {
  if (event.data && event.data.type === 'SKIP_WAITING') {
    self.skipWaiting()
  }
})

// Update check on install
self.addEventListener('install', () => {
  self.skipWaiting()
})

// Activate and claim clients
self.addEventListener('activate', (event) => {
  event.waitUntil(self.clients.claim())
})
