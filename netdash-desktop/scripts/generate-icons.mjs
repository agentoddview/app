import { promises as fs } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import pngToIco from 'png-to-ico';
import sharp from 'sharp';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

const iconDir = path.join(rootDir, 'icons');
const svgPath = path.join(iconDir, 'vite.svg');
const pngPath = path.join(iconDir, 'icon.png');
const icoPath = path.join(iconDir, 'icon.ico');

const sizes = [16, 24, 32, 48, 64, 128, 256, 1024];

async function ensureDir(target) {
  await fs.mkdir(target, { recursive: true });
}

async function generate() {
  await ensureDir(iconDir);
  const pngBuffers = [];

  for (const size of sizes) {
    const sizeFile = path.join(iconDir, `icon-${size}.png`);
    const buffer = await sharp(svgPath)
      .resize(size, size, { fit: 'cover' })
      .png()
      .toBuffer();

    await fs.writeFile(sizeFile, buffer);
    if (size === 1024) {
      await fs.writeFile(pngPath, buffer);
    }
    if (size <= 256) {
      pngBuffers.push(buffer);
    }
  }

  const ico = await pngToIco(pngBuffers);
  await fs.writeFile(icoPath, ico);
}

generate().catch((error) => {
  console.error('icon generation failed', error);
  process.exitCode = 1;
});
