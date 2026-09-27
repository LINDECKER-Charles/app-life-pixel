// Rasterize the canonical vector mark; no hand-maintained bitmap or new dependency is needed.
// Run from the repository root after installing the frontend's Playwright Chromium browser.
import { Buffer } from 'node:buffer';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';

const require = createRequire(new URL('../../frontend/package.json', import.meta.url));
const { chromium } = require('playwright');
const ROOT = new URL('../../', import.meta.url);
const ICON_BACKGROUND = '#fff8f5';
const FAVICON_SIZES = [16, 32, 48];
const DESKTOP_SIZES = [16, 32, 48, 64, 128, 256, 512, 1024];
const ICNS_TYPES = new Map([
  [16, 'icp4'],
  [32, 'icp5'],
  [64, 'icp6'],
  [128, 'ic07'],
  [256, 'ic08'],
  [512, 'ic09'],
  [1024, 'ic10'],
]);
const mark = await readFile(new URL('design-system/assets/mark.svg', ROOT), 'utf8');
const paths = mark.slice(mark.indexOf('>') + 1, mark.lastIndexOf('</svg>')).trim();

function iconSvg(size, inset) {
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${size} ${size}">
  <rect width="${size}" height="${size}" rx="${size / 5}" fill="${ICON_BACKGROUND}"/>
  <g transform="translate(${inset} ${inset})" shape-rendering="crispEdges">
    ${paths}
  </g>
</svg>\n`;
}

async function save(path, bytes) {
  const url = new URL(path, ROOT);
  await mkdir(new URL('.', url), { recursive: true });
  await writeFile(url, bytes);
}

async function rasterize(page, request) {
  const dataUrl = await page.evaluate(async ({ svg, size }) => {
    const image = new Image();
    image.src = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
    await image.decode();
    const canvas = document.createElement('canvas');
    canvas.width = size;
    canvas.height = size;
    const context = canvas.getContext('2d');
    context.imageSmoothingEnabled = false;
    context.drawImage(image, 0, 0, size, size);
    return canvas.toDataURL('image/png');
  }, request);
  return Buffer.from(dataUrl.split(',')[1], 'base64');
}

function ico(images) {
  const header = Buffer.alloc(6 + 16 * images.length);
  header.writeUInt16LE(1, 2);
  header.writeUInt16LE(images.length, 4);
  let offset = header.length;
  images.forEach(({ size, png }, index) => {
    const entry = 6 + index * 16;
    header.writeUInt8(size === 256 ? 0 : size, entry);
    header.writeUInt8(size === 256 ? 0 : size, entry + 1);
    header.writeUInt16LE(1, entry + 4);
    header.writeUInt16LE(32, entry + 6);
    header.writeUInt32LE(png.length, entry + 8);
    header.writeUInt32LE(offset, entry + 12);
    offset += png.length;
  });
  return Buffer.concat([header, ...images.map(({ png }) => png)]);
}

function icns(images) {
  const chunks = images
    .filter(({ size }) => ICNS_TYPES.has(size))
    .map(({ size, png }) => {
      const header = Buffer.alloc(8);
      header.write(ICNS_TYPES.get(size));
      header.writeUInt32BE(png.length + 8, 4);
      return Buffer.concat([header, png]);
    });
  const header = Buffer.alloc(8);
  header.write('icns');
  header.writeUInt32BE(8 + chunks.reduce((total, chunk) => total + chunk.length, 0), 4);
  return Buffer.concat([header, ...chunks]);
}

async function writeWebIcons(page, favicon, applicationIcon) {
  const images = [];
  for (const size of FAVICON_SIZES) {
    images.push({ size, png: await rasterize(page, { svg: favicon, size }) });
  }
  const touchIcon = await rasterize(page, { svg: applicationIcon, size: 180 });
  for (const project of ['app', 'admin']) {
    const directory = `frontend/projects/${project}/public/`;
    await save(`${directory}favicon.svg`, favicon);
    await save(`${directory}favicon.ico`, ico(images));
    await save(`${directory}apple-touch-icon.png`, touchIcon);
  }
}

async function writeDesktopIcons(page, applicationIcon) {
  const images = [];
  for (const size of DESKTOP_SIZES) {
    images.push({
      size,
      png: await rasterize(page, { svg: applicationIcon, size }),
    });
  }
  for (const { size, png } of images) {
    if ([32, 64, 128].includes(size)) await save(`tauri/icons/${size}x${size}.png`, png);
    if (size === 256) await save('tauri/icons/128x128@2x.png', png);
    if (size === 512) await save('tauri/icons/icon.png', png);
    if (size === 1024) await save('tauri/icons/source.png', png);
  }
  await save('tauri/icons/icon.ico', ico(images.filter(({ size }) => size <= 256)));
  await save('tauri/icons/icon.icns', icns(images));
}

const favicon = iconSvg(32, 0);
const applicationIcon = iconSvg(40, 4);
const browser = await chromium.launch({
  headless: true,
  executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE,
});
try {
  const page = await browser.newPage();
  await save('design-system/assets/identity/favicon.svg', favicon);
  await save('design-system/assets/identity/app-icon.svg', applicationIcon);
  await writeWebIcons(page, favicon, applicationIcon);
  await writeDesktopIcons(page, applicationIcon);
  console.log('Generated web and desktop icons from design-system/assets/mark.svg.');
} finally {
  await browser.close();
}
