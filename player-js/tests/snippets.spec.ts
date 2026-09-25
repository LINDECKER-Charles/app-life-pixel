import { readdir, readFile } from 'node:fs/promises';

import { expect, test } from '@playwright/test';

import { BLUE, MASCOT } from './fixtures/animations';
import { fakeExport } from './fixtures/fake-export';
import { PlayerPage } from './fixtures/player-page';

const SNIPPETS = new URL('../snippets/', import.meta.url);
const PLACEHOLDER = /\{\{(\w+)\}\}/g;

/** The placeholders of each template: the contract `compiler` renders. */
const PLACEHOLDERS: Record<string, string[]> = {
  'angular.ts': ['alt', 'className', 'src', 'tagAttribute'],
  'html.html': ['alt', 'loader', 'src', 'tagAttribute'],
  'react.tsx': ['altExpression', 'className', 'src', 'tagAttribute'],
  'vue.vue': ['alt', 'src', 'tagAttribute'],
};

function snippet(name: string): Promise<string> {
  return readFile(new URL(name, SNIPPETS), 'utf8');
}

function render(template: string, values: Record<string, string>): string {
  return template.replace(PLACEHOLDER, (_, name: string) => values[name] ?? '');
}

test('each template uses exactly its placeholders', async () => {
  const names = (await readdir(SNIPPETS)).sort();
  expect(names).toEqual(Object.keys(PLACEHOLDERS));
  for (const name of names) {
    const used = [...(await snippet(name)).matchAll(PLACEHOLDER)].map((match) => match[1]);
    expect([...new Set(used)].sort(), name).toEqual(PLACEHOLDERS[name]);
  }
});

test('the rendered HTML snippet plays the tag with its accessible name', async ({ page }) => {
  const body = render(await snippet('html.html'), {
    loader: '/life-pixel.js',
    src: '/mascot.wasm',
    tagAttribute: ' tag="jump"',
    alt: 'The mascot jumping',
  });
  const player = new PlayerPage(page);
  await player.open(body.replace('<life-pixel', '<life-pixel id="mascot"'), {
    '/mascot.wasm': { body: await fakeExport(MASCOT) },
  });

  await expect.poll(() => player.events()).toEqual(['mascot:load']);
  expect(await player.shownColor('mascot')).toEqual(BLUE);
  await expect(page.getByRole('img')).toHaveAccessibleName('The mascot jumping');
});
