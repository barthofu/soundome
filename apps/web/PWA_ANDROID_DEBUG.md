# PWA Android Troubleshooting Guide

## Prerequisites for Android Installation

Chrome on Android has strict requirements for PWA installation:

### 1. **HTTPS or Localhost Only**
- ❌ HTTP with IP address (`http://192.168.x.x`) - **WILL NOT INSTALL**
- ✅ HTTPS - Works
- ✅ `localhost:port` - Works
- ✅ `127.0.0.1:port` - Works

**Solution for local testing:**
Use a tunnel or SSH port forward to localhost, or deploy to a server with HTTPS.

### 2. **Service Worker Must Be Registered**
- Visit the page once so Service Worker registers
- Check DevTools → Application → Service Workers
- Wait a few seconds after first visit

### 3. **Manifest Must Be Valid**
- Must have `display: standalone` ✓
- Must have at least one icon ✓
- Icons must be accessible and loadable ✓
- `theme_color` must be set ✓
- `name` and `short_name` must be present ✓

## Testing on Android

### Step 1: Build for Production
```bash
cd apps/web
pnpm run build
```

### Step 2: Serve via HTTPS or Tunnel

Option A - Using ngrok (easiest for testing):
```bash
# Install if needed
npm install -g ngrok

# Start the server
cd apps/web
pnpm run preview
# Note the localhost URL (usually http://localhost:5000)

# In another terminal, tunnel it
ngrok http 5000
# Get the HTTPS URL like https://abc123.ngrok.io
```

Option B - Using SSH tunnel to a remote HTTPS server:
```bash
# Forward remote port to localhost
ssh -L 8443:soundome.example.com:443 user@server
# Then visit https://localhost:8443
```

### Step 3: Test on Android Chrome

1. **Connect Android phone to same network or use tunnel**

2. **Open Chrome and visit the HTTPS URL**
   - Wait for Service Worker to register (5-10 seconds)
   - Look for **"Install app"** button in address bar
   - If no button appears, see troubleshooting below

3. **Click Install app** → Choose name → Install

4. **App should appear** in app drawer

## Verification in DevTools

### On Android Chrome:
1. Open the URL on phone
2. Press `...` (menu) → **DevTools**
3. Go to **Application** tab

Check:
- ✓ Service Workers: `/sw.js` should be registered/active
- ✓ Manifest: Should load without errors, all icons present
- ✓ Cache Storage: Should see caches like `api-cache`, `local-api-cache`

### On Desktop (Android Emulator alternative):
1. Use Chrome DevTools → **Emulation** to simulate Android
2. Check same Application tab
3. Won't show install button in emulation, but SW/manifest still work

## Troubleshooting

### No "Install app" Button Appears

**Possible causes:**

1. **Not HTTPS/localhost**
   ```
   ❌ http://192.168.1.100:5000 - Won't install
   ✅ https://example.com - Will install
   ✅ http://localhost:5000 - Will install
   ```
   **Fix:** Use HTTPS or localhost tunnel

2. **Service Worker not registered**
   - DevTools → Application → Service Workers
   - Should show `/sw.js` with green dot
   - If empty: refresh page, wait 5-10 seconds
   - Check console for errors
   ```javascript
   // In console:
   navigator.serviceWorker.getRegistrations()
   ```

3. **Manifest not loading**
   - DevTools → Application → Manifest
   - Should show full manifest with icons
   - If error: manifest.webmanifest failed to load (404)
   - Check that `/manifest.webmanifest` is accessible

4. **Missing required manifest fields**
   - Must have `display: standalone`
   - Must have at least 2 icons (192×192 and 512×512)
   - Must have `name` and `description`
   - **Check:** `curl https://your-url/manifest.webmanifest`

### Installation Works but Logo Doesn't Appear

1. **Icons not found (404)**
   - DevTools → Network tab
   - Look for failed requests to `pwa-*.png`
   - **Fix:** Verify icons are in public folder and built

2. **Wrong icon size or format**
   - Android needs at least 192×192 and 512×512
   - PNG format required
   - **Check:** Icons in manifest match actual files

3. **Maskable icons not applied**
   - Android 8+ uses maskable icons if available
   - Icons must have `"purpose":"maskable"`
   - **Check:** manifest has `pwa-*-maskable.png` entries

### Installation Works but App Crashes/Blank

1. **Service Worker issues**
   - DevTools → Application → Service Workers
   - Check for red X or inactive status
   - Click "Update on reload" to refresh
   - Check console for JS errors

2. **Cache corruption**
   - DevTools → Application → Storage → Clear site data
   - Uninstall app, clear cache, reinstall
   - Or: Menu → Settings → Apps → Soundome → Storage → Clear data

3. **API routing issues**
   - App can't reach backend API
   - Check Network tab for failed requests
   - Verify API base URL is correct
   - For local dev: API must be accessible from phone/tunnel URL

## Verification Checklist

Before testing on Android:

- [ ] Built with `pnpm run build`
- [ ] Serving via HTTPS or localhost
- [ ] Service Worker registered (check DevTools)
- [ ] Manifest loads (no 404 errors)
- [ ] Icons load (Network tab shows 200)
- [ ] No console errors in app
- [ ] API is accessible from device

## Quick Test Commands

```bash
# Verify manifest exists and is valid
curl https://your-url/manifest.webmanifest | jq .

# Verify service worker is served
curl -I https://your-url/sw.js

# Verify icons are accessible
curl -I https://your-url/pwa-192x192.png
curl -I https://your-url/pwa-512x512.png

# Check for JSON errors
cat data/web/manifest.webmanifest | jq .
```

## Browser Compatibility

| Browser | Status | Android Install |
|---------|--------|-----------------|
| Chrome | ✅ Full | ✅ Yes |
| Edge | ✅ Full | ✅ Yes |
| Firefox | ✅ Full | ⚠️ Limited |
| Samsung Internet | ✅ Full | ✅ Yes |
| UC Browser | ⚠️ Partial | ❌ No |

## Common Manifest Issues

```json
// ❌ WRONG - Missing key fields
{
  "name": "Soundome"
}

// ✅ CORRECT - All required fields
{
  "name": "Soundome",
  "short_name": "Soundome",
  "description": "Personal music library manager",
  "start_url": "/",
  "display": "standalone",
  "scope": "/",
  "theme_color": "#6366f1",
  "background_color": "#ffffff",
  "icons": [
    {
      "src": "pwa-192x192.png",
      "sizes": "192x192",
      "type": "image/png",
      "purpose": "any"
    },
    {
      "src": "pwa-512x512.png",
      "sizes": "512x512",
      "type": "image/png",
      "purpose": "any"
    },
    {
      "src": "pwa-192x192-maskable.png",
      "sizes": "192x192",
      "type": "image/png",
      "purpose": "maskable"
    },
    {
      "src": "pwa-512x512-maskable.png",
      "sizes": "512x512",
      "type": "image/png",
      "purpose": "maskable"
    }
  ]
}
```

## Still Not Working?

1. **Test with Lighthouse**
   - Chrome DevTools → Lighthouse → PWA audit
   - Shows all missing requirements
   - Most reliable PWA checker

2. **Check PWA Builder**
   - Visit https://www.pwabuilder.com/
   - Enter your URL
   - Generates detailed PWA report

3. **Verify on Different Device**
   - Try different phone/tablet
   - Try different Chrome version
   - Try Chrome Beta

4. **Check Server Logs**
   - Look for 404 errors on manifest/icons
   - Verify HTTPS certificate is valid
   - Check that server sends correct MIME types

## Reference

- [MDN - Progressive Web Apps](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps)
- [Web.dev - PWA Checklist](https://web.dev/progressive-web-apps/)
- [Android Chrome PWA Install](https://developer.chrome.com/docs/android/trusted-web-activity/)
- [W3C Web App Manifest](https://w3c.github.io/manifest/)
