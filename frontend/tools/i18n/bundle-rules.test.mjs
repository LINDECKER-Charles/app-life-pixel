import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { findBundledEntries } from './bundle-rules.mjs';

describe('findBundledEntries', () => {
  const keys = ['app.name', 'common.close'];

  it('finds a catalogue compiled into a script, as an object or as JSON text', () => {
    const scripts = [
      { name: 'main.js', text: 'const c={"app.name":"Life Pixel"};' },
      { name: 'chunk.js', text: `JSON.parse('{\\"common.close\\": \\"Close\\"}')` },
    ];
    assert.deepEqual(findBundledEntries(scripts, keys), [
      'main.js: app.name',
      'chunk.js: common.close',
    ]);
  });

  it('accepts keys used to translate', () => {
    const scripts = [{ name: 'main.js', text: 'e.translate("common.close");t={key:"app.name"}' }];
    assert.deepEqual(findBundledEntries(scripts, keys), []);
  });

  it('ignores a minified ternary between two backtick key literals', () => {
    const scripts = [{ name: 'chunk.js', text: 'm===`edit`?`common.close`:`app.name`' }];
    assert.deepEqual(findBundledEntries(scripts, keys), []);
  });
});
