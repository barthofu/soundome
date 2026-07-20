# PWA Quick Start Guide

## What is a PWA?

A Progressive Web App (PWA) is a web application that works like a native app:
- Can be installed on devices
- Works offline
- Loads fast
- Sends notifications
- Gets automatic updates

## Testing PWA Features

### Start Development Server
```bash
cd apps/web
pnpm run dev
# Open http://localhost:5173
```

### Build for Production
```bash
cd apps/web
pnpm run build
pnpm run preview
# Open the preview URL (usually http://localhost:5000)
```

## DevTools Testing (Chrome/Edge)

1. **Press F12** to open DevTools
2. **Go to Application tab**
3. **Service Workers section:**
   - See `/sw.js` registered ✓
   - Check "Offline" to test offline mode
4. **Manifest section:**
   - See `manifest.json` loaded
   - View all app icons
5. **Cache Storage:**
   - See what assets are cached
   - See what API responses are cached

## Testing Installation

### Desktop
- Look for "Install app" button in address bar
- Or: Menu (⋮) → "Install Soundome"

### Mobile (Android)
- Menu (⋮) → "Install app"
- Or: Long-press → "Install"

### iOS (Safari)
- Share button → "Add to Home Screen"

## Testing Offline

1. Open DevTools (F12)
2. Go to Application → Service Workers
3. Check "Offline"
4. Refresh page
5. App still works (with cached data)

## What's Cached?

| Type | Cache Strategy | Expiry |
|------|----------------|--------|
| JS, CSS, Images | Cache First | 1 week |
| API calls | Network First | 5 min |
| Pages | Network First | - |

**Network First** = Try online first, use cache if offline
**Cache First** = Use cache, only fetch if missing

## File Structure

```
apps/web/
├── src/
│   ├── sw.ts                    ← Service Worker
│   ├── lib/pwa.ts               ← PWA utilities
│   ├── components/
│   │   └── PWAUpdatePrompt.svelte ← Update UI
│   ├── main.ts                  ← Calls initPWA()
│   └── App.svelte               ← Uses PWAUpdatePrompt
├── scripts/
│   └── generate-pwa-icons.mjs   ← Icon generator
├── public/
│   ├── pwa-192x192.png
│   ├── pwa-512x512.png
│   ├── pwa-192x192-maskable.png
│   └── pwa-512x512-maskable.png
├── index.html                   ← PWA meta tags
├── vite.config.ts               ← PWA config
└── PWA.md                        ← Full docs
```

## Key Files

### sw.ts (Service Worker)
- Handles caching strategy
- Intercepts network requests
- Manages updates

### lib/pwa.ts (PWA Utils)
- `initPWA()` - Start the service worker
- `refreshPWA()` - Request update
- `onPWAUpdate()` - Listen for updates
- `isPWAAvailable()` - Check support

### PWAUpdatePrompt.svelte (UI)
- Shows update notification
- Handles update click
- Mobile responsive

## Common Tasks

### Check if PWA is registered
```javascript
// In browser console
navigator.serviceWorker.getRegistrations()
```

### View cached files
```javascript
// In browser console
await caches.keys() // Get all cache names
const cache = await caches.open('assets-cache')
await cache.keys() // View cached files
```

### Clear all caches
```javascript
// In DevTools Console
caches.keys().then(names => {
  names.forEach(name => caches.delete(name))
})
```

### Force service worker update check
```bash
# Refresh twice quickly, or:
# Click "Update" button when prompt appears
```

## Troubleshooting

### Service Worker not appearing
- Ensure you're on `localhost` or HTTPS
- Hard refresh: Ctrl+Shift+R
- Clear DevTools cache
- Check browser console for errors

### App not installable
- Must be HTTPS (or localhost)
- Manifest must be valid
- Service worker must be registered
- Visit site once to register SW

### Cache issues
- Clear DevTools → Application → Clear site data
- Or: Settings → Privacy → Clear browsing data → Cookies

### Updates not showing
- Wait 60 seconds (update check interval)
- Force refresh page
- Service Worker checks: App → Settings → About

## Performance

- Offline access works even after browser restart
- Assets cached for 1 week (or until app updated)
- API responses cached for 5 minutes
- Total precache size: ~580 KB (includes all assets)

## Browser Support

| Browser | Status | Notes |
|---------|--------|-------|
| Chrome | ✅ | Full support, best testing |
| Edge | ✅ | Full support |
| Firefox | ✅ | Full support |
| Safari | ⚠️ | iOS 16.4+, limited features |
| Opera | ✅ | Full support |

## Next Steps

1. ✅ Test offline mode
2. ✅ Test installation
3. ✅ Test update prompt
4. ✅ Test cache storage
5. Consider: Background sync, notifications, shortcuts

## Links

- [Full PWA Documentation](./PWA.md)
- [Vite PWA Plugin Docs](https://vite-pwa-org.netlify.app/)
- [Web.dev PWA Guide](https://web.dev/progressive-web-apps/)
- [MDN Service Workers](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API)
