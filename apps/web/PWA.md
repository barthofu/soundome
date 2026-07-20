# Soundome Web Frontend - PWA Support

This frontend application includes full Progressive Web App (PWA) support, allowing it to work offline and be installed as a native-like application.

## Features

- ✅ **Offline Support**: Works offline with cached assets and API responses
- ✅ **App Installation**: Can be installed on mobile and desktop devices
- ✅ **Auto-Update**: Automatically checks for updates every 60 seconds
- ✅ **Caching Strategy**: Smart caching for assets, API calls, and pages
- ✅ **Apple/iOS Support**: Support for iOS home screen installation

## How PWA Works

### Service Worker
The PWA uses a Service Worker (`src/sw.ts`) to:
- Cache static assets (JS, CSS, images)
- Intercept API calls with Network First strategy
- Enable offline browsing of cached pages
- Auto-update when new versions are available

### Manifest
The `manifest.json` (generated in `public/manifest.json`) defines:
- App name, icon, and colors
- Display mode (standalone app)
- Installation scope and start URL

### Icons
PWA icons are generated from the SVG favicon:
- **pwa-192x192.png**: Standard app icon
- **pwa-512x512.png**: Large app icon for splash screens
- **pwa-192x192-maskable.png**: Icon for masking (adaptive icons)
- **pwa-512x512-maskable.png**: Large maskable icon

## Development

### Building Icons
Icons are generated automatically during `npm run dev` and `npm run build`:

```bash
pnpm run generate-icons
```

This converts `public/favicon.svg` to PNG files required for PWA.

### Local Testing

1. **Build the app**:
   ```bash
   pnpm run build
   ```

2. **Serve locally** (using a tool like `serve`):
   ```bash
   npx serve -s ../../data/web
   ```

3. **Check PWA**:
   - Open Chrome DevTools → Application tab → Service Workers
   - Check "Offline" to test offline functionality
   - Use Chrome's "Install app" option to test installation

### Update Prompts
When a new version is available, users see an update prompt with options to:
- **Update Now**: Refresh to the latest version
- **Later**: Dismiss and update on next page load

## Configuration

### Caching Strategy

The Service Worker uses different strategies for different content:

| Content | Strategy | Expiry |
|---------|----------|--------|
| API calls | Network First | 5 minutes |
| Static assets | Cache First | 1 week |
| Pages | Network First | - |

### Manifest Configuration
Edit `vite.config.ts` to customize:
- App name and description
- Theme colors
- Display mode
- Icon purposes

## Browser Support

| Browser | Support | Notes |
|---------|---------|-------|
| Chrome/Edge | ✅ Full | Desktop & Mobile |
| Firefox | ✅ Full | Desktop & Mobile |
| Safari | ⚠️ Limited | iOS 16.4+, macOS 16.4+ |
| Opera | ✅ Full | Desktop & Mobile |

## Troubleshooting

### Service Worker not registering
- Check browser DevTools → Application → Service Workers
- Ensure HTTPS (or localhost for dev)
- Check browser console for errors

### Not installing as app
- Must have HTTPS (except localhost)
- Check manifest validity: Add to home screen / Install
- Icons must be served correctly

### Offline not working
- Check DevTools → Application → Cache Storage
- Verify routes are cached
- Check Service Worker → Offline mode in DevTools

### Updates not appearing
- Service Worker checks every 60 seconds
- Force refresh (Ctrl+Shift+R) to check immediately
- Clear site data in DevTools if needed

## Future Enhancements

- [ ] Background sync for offline actions
- [ ] Push notifications for playlist updates
- [ ] Periodic background sync for library refresh
- [ ] App shortcuts for common actions
- [ ] File handling for drag-and-drop uploads
