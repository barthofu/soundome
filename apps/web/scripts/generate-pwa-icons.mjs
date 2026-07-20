#!/usr/bin/env node

import sharp from 'sharp'
import fs from 'fs/promises'
import path from 'path'

const publicDir = path.resolve(process.cwd(), 'public')

async function generateIcons() {
  console.log('Generating PWA icons...')

  // SVG source - create a simple PNG from the favicon
  const svgBuffer = await fs.readFile(path.join(publicDir, 'favicon.svg'))

  // Generate 192x192 icon
  await sharp(svgBuffer)
    .resize(192, 192, {
      fit: 'contain',
      background: { r: 0, g: 0, b: 0, alpha: 0 },
    })
    .png()
    .toFile(path.join(publicDir, 'pwa-192x192.png'))
  console.log('✓ Generated pwa-192x192.png')

  // Generate 512x512 icon
  await sharp(svgBuffer)
    .resize(512, 512, {
      fit: 'contain',
      background: { r: 0, g: 0, b: 0, alpha: 0 },
    })
    .png()
    .toFile(path.join(publicDir, 'pwa-512x512.png'))
  console.log('✓ Generated pwa-512x512.png')

  // Generate 192x192 maskable icon (with padding)
  await sharp(svgBuffer)
    .resize(168, 168, {
      fit: 'contain',
      background: { r: 0, g: 0, b: 0, alpha: 0 },
    })
    .extend({
      top: 12,
      bottom: 12,
      left: 12,
      right: 12,
      background: { r: 0, g: 0, b: 0, alpha: 0 },
    })
    .png()
    .toFile(path.join(publicDir, 'pwa-192x192-maskable.png'))
  console.log('✓ Generated pwa-192x192-maskable.png')

  // Generate 512x512 maskable icon (with padding)
  await sharp(svgBuffer)
    .resize(448, 448, {
      fit: 'contain',
      background: { r: 0, g: 0, b: 0, alpha: 0 },
    })
    .extend({
      top: 32,
      bottom: 32,
      left: 32,
      right: 32,
      background: { r: 0, g: 0, b: 0, alpha: 0 },
    })
    .png()
    .toFile(path.join(publicDir, 'pwa-512x512-maskable.png'))
  console.log('✓ Generated pwa-512x512-maskable.png')

  console.log('PWA icons generated successfully!')
}

generateIcons().catch((err) => {
  console.error('Error generating icons:', err)
  process.exit(1)
})
