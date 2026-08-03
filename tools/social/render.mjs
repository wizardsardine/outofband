// Regenerates the social card and the raster icons, so neither is a binary
// nobody can reproduce. Both are screenshots taken by a headless Chromium:
// the card of `og-template.html`, the icons of `assets/favicon.svg`, which
// keeps every raster derived from a source in this repository.
//
//   npm install && node render.mjs
//
// PLAYWRIGHT_CHROMIUM points at an existing browser if you have one and
// would rather not have playwright download another.
import { chromium } from 'playwright';
import { readFileSync, writeFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, resolve } from 'path';

const HERE = dirname(fileURLToPath(import.meta.url));
const ASSETS = resolve(HERE, '../../crates/broadcast-frontend/assets');
const launch = process.env.PLAYWRIGHT_CHROMIUM
  ? { executablePath: process.env.PLAYWRIGHT_CHROMIUM }
  : {};

const browser = await chromium.launch(launch);

// The card, at the 1200x630 both Open Graph and Twitter want. The template
// asks for the page's own woff2 faces by a placeholder, because a relative
// @font-face URL would resolve against wherever the render file happens to
// sit; substituting an absolute file:// keeps the card in the page's fonts
// without a download.
const template = readFileSync(resolve(HERE, 'og-template.html'), 'utf8');
const rendered = resolve(HERE, 'og-render.html');
writeFileSync(rendered, template.replaceAll('FONTDIR', `file://${ASSETS}/fonts`));

const card = await browser.newPage({
  viewport: { width: 1200, height: 630 },
  deviceScaleFactor: 1,
});
await card.goto(`file://${rendered}`, { waitUntil: 'networkidle' });
await card.waitForTimeout(600); // let the woff2 faces paint before capture
await card.screenshot({ path: `${ASSETS}/og.png` });
await card.close();

// The icons, from the same SVG the page links directly, so the raster
// fallbacks cannot drift from the vector. The markup is inlined rather than
// loaded as <img src="file://...">: a setContent page has an opaque origin
// and is refused local files.
const svg = readFileSync(`${ASSETS}/favicon.svg`, 'utf8');
for (const [size, name] of [
  [180, 'apple-touch-icon'],
  [32, 'favicon-32'],
]) {
  const page = await browser.newPage({
    viewport: { width: size, height: size },
    deviceScaleFactor: 1,
  });
  await page.setContent(
    `<html><body style="margin:0;width:${size}px;height:${size}px">${svg}</body></html>`,
    { waitUntil: 'load' },
  );
  await page.addStyleTag({ content: `svg{width:${size}px;height:${size}px;display:block}` });
  await page.waitForTimeout(200);
  await page.screenshot({ path: `${ASSETS}/${name}.png` });
  await page.close();
}

await browser.close();
console.log(`wrote og.png, apple-touch-icon.png and favicon-32.png to ${ASSETS}`);
