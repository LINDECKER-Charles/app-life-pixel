// Keeps the production tokens and the design system's from drifting apart (plan C1): every colour
// of _tokens.scss equals its role in design-system/tokens/tokens.css, light, dark and forced.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { describe, it } from 'node:test';
import { alpha, blockAfter, channels, customProperties } from './css-blocks.mjs';
import { PRODUCTION_ONLY, RENDERER_CONSTANTS, TOKEN_ROLES } from './token-roles.mjs';

const read = (path) => readFileSync(new URL(path, import.meta.url), 'utf8');
const SCSS = read('../../projects/shared/src/styles/_tokens.scss');
const CSS = read('../../../design-system/tokens/tokens.css');
const RENDERER = read('../../projects/app/src/app/canvas/render/canvas-renderer.ts');

const merged = (...maps) => new Map(maps.flatMap((map) => [...map]));
const productionRoot = customProperties(blockAfter(SCSS, '\n:root {'));
const designLight = customProperties(blockAfter(CSS, ":root[data-theme='light']"));
const production = {
  light: merged(productionRoot, customProperties(blockAfter(SCSS, '@mixin light-colors'))),
  dark: merged(productionRoot, customProperties(blockAfter(SCSS, '@mixin dark-colors'))),
  forced: customProperties(blockAfter(SCSS, '@mixin forced-colors')),
};
const design = {
  light: designLight,
  dark: merged(designLight, customProperties(blockAfter(CSS, ":root[data-theme='dark']"))),
  forced: customProperties(blockAfter(blockAfter(CSS, '@media (forced-colors: active)'), ':root')),
};

const isColourToken = (name) => /^--lp-(color|shadow)-/.test(name) && !name.endsWith('-rgb');
const isDesignColour = (name, value) => /^(#|rgb\()/.test(value) || name.startsWith('--lp-shadow');

function mismatches(theme) {
  const differences = {};
  for (const [name, role] of Object.entries(TOKEN_ROLES)) {
    const [actual, expected] = [production[theme].get(name), design[theme].get(role)];
    if (actual !== expected) differences[`${name} (${role})`] = { actual, expected };
  }
  return differences;
}

describe('the colour tokens of production and of the design system', () => {
  it('give every production colour a role, or a stated reason to have none', () => {
    const names = new Set([...production.light.keys(), ...production.dark.keys()]);
    const unmapped = [...names].filter(
      (name) => isColourToken(name) && !(name in TOKEN_ROLES) && !(name in PRODUCTION_ONLY),
    );

    assert.deepEqual(unmapped, []);
  });

  it('use every colour role of the design system', () => {
    const used = new Set([...Object.values(TOKEN_ROLES), ...Object.values(RENDERER_CONSTANTS)]);
    const unused = [...design.light].filter(
      ([name, value]) => isDesignColour(name, value) && !used.has(name),
    );

    assert.deepEqual(unused, []);
  });

  for (const theme of ['light', 'dark']) {
    it(`match role by role in the ${theme} theme`, () => {
      assert.deepEqual(mismatches(theme), {});
    });

    it(`give Ionic the same channels as each colour in the ${theme} theme`, () => {
      const colours = production[theme];
      for (const [name, value] of colours) {
        if (!name.endsWith('-rgb')) continue;
        const base = colours.get(name.slice(0, -'-rgb'.length));
        assert.deepEqual(channels(value), channels(base), name);
      }
      assert.equal(
        Number(colours.get('--lp-scrim-opacity')),
        alpha(colours.get('--lp-color-scrim')),
      );
    });
  }

  it('map the same roles onto the same system colours under forced colours', () => {
    const byRole = Object.fromEntries(
      [...production.forced].map(([name, value]) => [TOKEN_ROLES[name] ?? name, value]),
    );

    assert.deepEqual(byRole, Object.fromEntries(design.forced));
  });

  it('apply the dark set to an explicit dark theme and to a dark system theme', () => {
    const explicit = blockAfter(SCSS, ":root[data-theme='dark']");
    const system = blockAfter(SCSS, '@media (prefers-color-scheme: dark)');

    assert.match(explicit, /@include dark-colors;/);
    assert.match(system, /:root:not\(\[data-theme='light'\]\)\s*\{\s*@include dark-colors;/);
  });

  it('draw the canvas with the neutral checkerboard and grid of the design system', () => {
    for (const [constant, role] of Object.entries(RENDERER_CONSTANTS)) {
      const match = new RegExp(`const ${constant} = '([^']+)';`).exec(RENDERER);
      assert.equal(match?.[1], design.light.get(role), constant);
      assert.equal(design.dark.get(role), design.light.get(role), `${role} ignores the theme`);
    }
  });
});
