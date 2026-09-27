// Optional reference verification; reuses the frontend workspace's Playwright installation.
import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

const require = createRequire(new URL('../../frontend/package.json', import.meta.url));
const { chromium } = require('playwright');
const BASE = process.env.PREVIEW_URL ?? 'http://127.0.0.1:4265/design-system/preview/';
const OUTPUT = new URL('../review/', import.meta.url);
const SECTIONS = ['overview', 'foundations', 'components', 'studio', 'journeys'];
const browser = await chromium.launch({
  headless: true,
  executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE,
});
const page = await browser.newPage({ viewport: { width: 1440, height: 1100 } });
const failures = [];
let checks = 0;

function verify(condition, message) {
  assert.ok(condition, message);
  checks += 1;
}

async function go(section) {
  await page.locator(`[data-section='${section}']`).click();
  await page.waitForFunction((id) => !document.getElementById(id).hidden, section);
}

async function capture(name) {
  await page.evaluate(() => window.scrollTo({ top: 0, behavior: 'instant' }));
  await page.screenshot({
    path: fileURLToPath(new URL(`${name}.png`, OUTPUT)),
    fullPage: true,
    animations: 'disabled',
  });
}

async function checkLayout(width) {
  await page.setViewportSize({ width, height: 1000 });
  for (const section of SECTIONS) {
    await go(section);
    const hasOverflow = await page.evaluate(
      () => document.documentElement.scrollWidth > window.innerWidth + 1,
    );
    verify(!hasOverflow, `No page overflow: ${section} at ${width}px`);
    verify((await page.locator('main > section:visible').count()) === 1, 'One section is visible');
    verify(await page.locator('main > section:visible h1').textContent(), 'Section is named');
  }
}

async function checkDialog() {
  await go('components');
  const opener = page.locator('[data-open-dialog]').first();
  await opener.click();
  verify(await page.locator('#create-dialog').isVisible(), 'Creation dialog opens');
  verify(
    await page.locator('#animation-name').evaluate((el) => el === document.activeElement),
    'Dialog focuses its input',
  );
  await page.locator('#create-form [type=submit]').click();
  verify(await page.locator('#name-error').isVisible(), 'Blank name reports a visible error');
  verify(
    (await page.locator('#animation-name').getAttribute('aria-invalid')) === 'true',
    'Blank name is marked invalid',
  );
  await page.keyboard.press('Escape');
  verify(!(await page.locator('#create-dialog').isVisible()), 'Escape closes dialog');
  verify(await opener.evaluate((el) => el === document.activeElement), 'Dialog restores focus');
}

async function checkCreation() {
  const opener = page.locator('[data-open-dialog]').first();
  await opener.click();
  await page.locator('#animation-name').fill('<Pip & friends>');
  await page.locator('#create-form [type=submit]').click();
  await page.waitForFunction(() => !document.querySelector('#studio').hidden);
  verify(
    (await page.locator('#demo-name').textContent()) === '<Pip & friends>',
    'User text stays literal and opens the studio',
  );
  verify((await page.locator('#demo-name').locator('*').count()) === 0, 'Name cannot inject HTML');
}

async function checkStudio() {
  await go('studio');
  await page.locator('[data-tool=eraser]').click();
  verify(
    (await page.locator('[data-tool=eraser]').getAttribute('aria-pressed')) === 'true',
    'Tool selection updates',
  );
  verify((await page.locator('[data-tool][aria-pressed=true]').count()) === 1, 'One selected tool');
  await page.locator('.swatch-mint').click();
  verify((await page.locator('.swatch[aria-pressed=true]').count()) === 1, 'One selected colour');
}

async function checkTimelineAndExport() {
  await page.locator('[data-frame="2"]').click();
  verify(
    (await page.locator('#canvas-pip').getAttribute('data-frame')) === '2',
    'Frame pose updates',
  );
  await page.locator('#zoom').selectOption('1.5');
  verify(
    (await page
      .locator('#canvas-pip')
      .evaluate((el) => el.style.getPropertyValue('--demo-zoom'))) === '1.5',
    'Zoom updates',
  );
  await page.locator('#export-button').click();
  verify(await page.locator('#export-options').isVisible(), 'Export panel opens');
  await page.locator('#export-format').selectOption('GIF');
  verify((await page.locator('#export-format').inputValue()) === 'GIF', 'Export format changes');
  await page.locator('#export-button').click();
}

async function checkLanguageFailure() {
  await page.route('**/i18n/en.json', (route) => route.abort());
  await page.locator('#language').selectOption('en');
  await page.waitForFunction(() => document.querySelector('#language').value === 'fr');
  verify(
    (await page.locator('html').getAttribute('lang')) === 'fr',
    'Failed locale keeps old content',
  );
  verify(await page.locator('#toast').isVisible(), 'Failed locale reports retry feedback');
  await page.unroute('**/i18n/en.json');
}

async function checkLanguageAndTheme() {
  await page.locator('#language').selectOption('en');
  await page.waitForFunction(() => document.documentElement.lang === 'en');
  verify(
    (await page.locator('#studio-title').textContent()) === 'The art comes first.',
    'English loaded at runtime',
  );
  verify(
    (await page.locator('#selected-tool').textContent()) === 'Eraser',
    'Selected tool translates',
  );
  verify(
    (await page.locator('#demo-name').textContent()) === '<Pip & friends>',
    'Locale switch preserves user input',
  );
  await page.locator('#theme').click();
  verify((await page.locator('html').getAttribute('data-theme')) === 'dark', 'Dark theme enabled');
  await capture('studio-dark');
  await page.locator('#theme').click();
  await page.locator('#language').selectOption('fr');
  await page.waitForFunction(() => document.documentElement.lang === 'fr');
}

async function checkKeyboardAndMotion() {
  await go('components');
  await page.locator('#reduce-motion').check();
  verify(
    (await page.locator('html').getAttribute('data-motion')) === 'reduce',
    'Manual reduced motion',
  );
  await page.locator('#reduce-motion').uncheck();
  await page.emulateMedia({ reducedMotion: 'reduce' });
  const duration = await page
    .locator('.button')
    .first()
    .evaluate((el) => getComputedStyle(el).transitionDuration);
  verify(
    duration.split(',').every((value) => parseFloat(value) <= 0.001),
    'OS reduced motion',
  );
  await page.emulateMedia({ reducedMotion: 'no-preference' });
  await page.locator('.skip-link').focus();
  await page.keyboard.press('Enter');
  verify(
    await page.locator('#main').evaluate((el) => el === document.activeElement),
    'Skip link focuses main',
  );
  verify(await page.locator('#components').isVisible(), 'Skip link keeps the current section');
}

page.on('pageerror', (error) => failures.push(error.message));
page.on('response', (response) => {
  if (response.status() >= 400) failures.push(`${response.status()} ${response.url()}`);
});
page.on('request', (request) => {
  if (new URL(request.url()).origin !== new URL(BASE).origin) failures.push('External request');
});

try {
  await mkdir(OUTPUT, { recursive: true });
  await page.goto(BASE);
  await page.waitForSelector('main[aria-busy=false]');
  await page.evaluate(() => document.fonts.ready);
  verify(
    await page
      .locator('[data-i18n]')
      .evaluateAll((elements) =>
        elements.every(
          (el) => el.textContent.trim() && !el.textContent.startsWith('design_system.'),
        ),
      ),
    'Every label translates',
  );
  await capture('overview-desktop');
  await checkDialog();
  await checkCreation();
  await checkStudio();
  await checkTimelineAndExport();
  await capture('studio-desktop');
  await checkLanguageFailure();
  await checkLanguageAndTheme();
  await checkKeyboardAndMotion();
  for (const width of [1440, 1024, 768, 390, 320]) await checkLayout(width);
  await page.setViewportSize({ width: 390, height: 844 });
  await go('overview');
  await capture('overview-mobile');
  await go('studio');
  await capture('studio-mobile');
  verify(failures.length === 0, failures.join('\n'));
  console.log(
    `PASS: ${checks} browser checks; 5 screenshots; no unexpected HTTP/page/external errors.`,
  );
} finally {
  await browser.close();
}
