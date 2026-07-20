#!/usr/bin/env node

import sharp from 'sharp'
import fs from 'fs/promises'
import path from 'path'

const publicDir = path.resolve(process.cwd(), 'public')
const logoPath = path.join(publicDir, 'logo_soundome.png')

async function generateAllIcons() {
  console.log('Generating all icons from logo_soundome.png...')

  // Read the logo
  const logoBuffer = await fs.readFile(logoPath)

  // Generate favicon.ico (32x32)
  await sharp(logoBuffer)
    .resize(32, 32, {
      fit: 'cover',
      background: { r: 0, g: 0, b: 0, alpha: 0 },
    })
    .png()
    .toFile(path.join(publicDir, 'favicon.png'))
  console.log('✓ Generated favicon.png (32x32)')

  // Generate iOS app icon (180x180)
  await sharp(logoBuffer)
    .resize(180, 180, {
      fit: 'cover',
      background: { r: 255, g: 255, b: 255, alpha: 1 },
    })
    .png()
    .toFile(path.join(publicDir, 'apple-touch-icon.png'))
  console.log('✓ Generated apple-touch-icon.png (180x180)')

  // Generate 192x192 icon
  await sharp(logoBuffer)
    .resize(192, 192, {
      fit: 'cover',
      background: { r: 0, g: 0, b: 0, alpha: 0 },
    })
    .png()
    .toFile(path.join(publicDir, 'pwa-192x192.png'))
  console.log('✓ Generated pwa-192x192.png (192x192)')

  // Generate 512x512 icon
  await sharp(logoBuffer)
    .resize(512, 512, {
      fit: 'cover',
      background: { r: 0, g: 0, b: 0, alpha: 0 },
    })
    .png()
    .toFile(path.join(publicDir, 'pwa-512x512.png'))
  console.log('✓ Generated pwa-512x512.png (512x512)')

  // Generate 192x192 maskable icon (with safe zone padding)
  const maskablePadding = 12
  await sharp(logoBuffer)
    .resize(192 - 2 * maskablePadding, 192 - 2 * maskablePadding, {
      fit: 'contain',
      background: { r: 0, g: 0, b: 0, alpha: 0 },
    })
    .extend({
      top: maskablePadding,
      bottom: maskablePadding,
      left: maskablePadding,
      right: maskablePadding,
      background: { r: 0, g: 0, b: 0, alpha: 0 },
    })
    .png()
    .toFile(path.join(publicDir, 'pwa-192x192-maskable.png'))
  console.log('✓ Generated pwa-192x192-maskable.png (192x192, maskable)')

  // Generate 512x512 maskable icon (with safe zone padding)
  const maskablePadding512 = 32
  await sharp(logoBuffer)
    .resize(512 - 2 * maskablePadding512, 512 - 2 * maskablePadding512, {
      fit: 'contain',
      background: { r: 0, g: 0, b: 0, alpha: 0 },
    })
    .extend({
      top: maskablePadding512,
      bottom: maskablePadding512,
      left: maskablePadding512,
      right: maskablePadding512,
      background: { r: 0, g: 0, b: 0, alpha: 0 },
    })
    .png()
    .toFile(path.join(publicDir, 'pwa-512x512-maskable.png'))
  console.log('✓ Generated pwa-512x512-maskable.png (512x512, maskable)')

  console.log('\n✓ All icons generated successfully!')
  console.log('  Total: 6 icon files generated')
}

generateAllIcons().catch((err) => {
  console.error('Error generating icons:', err)
  process.exit(1)
})
