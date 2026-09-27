import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { describe, it } from 'node:test';
import {
  MAX_SUBJECT_LENGTH,
  checkBranch,
  checkTitle,
  countReviewedLines,
  readScopes,
} from './pull-request-rules.mjs';

const CLAUDE_MD = readFileSync(new URL('../../CLAUDE.md', import.meta.url), 'utf8');
const SCOPES = ['core', 'player', 'ci'];

describe('readScopes', () => {
  it('reads every scope of the commit map of CLAUDE.md, once each', () => {
    const scopes = readScopes(CLAUDE_MD);

    for (const scope of ['core', 'player', 'editor-wasm', 'i18n', 'infra', 'ci', 'docs']) {
      assert.ok(scopes.includes(scope), scope);
    }
    assert.equal(new Set(scopes).size, scopes.length);
  });

  it('ignores arrows outside the commit section', () => {
    const text = '## other\n- `a/**` → `ghost`\n## commit\n- `b/**` → `real`\n## next\n';

    assert.deepEqual(readScopes(text), ['real']);
  });
});

describe('checkTitle', () => {
  it('accepts a subject with or without a scope, breaking or not', () => {
    for (const title of [
      'feat(core): add onion skin',
      'build: bump axum to 0.8.9',
      'fix(player)!: stop on the last frame',
    ]) {
      assert.deepEqual(checkTitle(title, SCOPES), [], title);
    }
  });

  it('refuses a free-form title', () => {
    assert.equal(checkTitle('Add onion skin', SCOPES).length, 1);
  });

  it('refuses an unknown type or scope', () => {
    assert.equal(checkTitle('feature(core): add onion skin', SCOPES).length, 1);
    assert.equal(checkTitle('feat(editor): add onion skin', SCOPES).length, 1);
  });

  it('refuses a capitalised description and a trailing period', () => {
    assert.equal(checkTitle('feat(core): Add onion skin.', SCOPES).length, 2);
  });

  it('refuses a subject longer than the limit', () => {
    const title = `feat(core): ${'a'.repeat(MAX_SUBJECT_LENGTH)}`;

    assert.equal(checkTitle(title, SCOPES).length, 1);
  });
});

describe('checkBranch', () => {
  it('accepts a short type and two to five kebab-case words', () => {
    assert.deepEqual(checkBranch('feat/onion-skin'), []);
    assert.deepEqual(checkBranch('fix/gif-palette-order-on-export'), []);
  });

  it('refuses a long type, a single word, too many words, or another case', () => {
    assert.equal(checkBranch('feature/onion-skin').length, 1);
    assert.equal(checkBranch('feat/onion').length, 1);
    assert.equal(checkBranch('feat/a-b-c-d-e-f').length, 1);
    assert.equal(checkBranch('feat/Onion_Skin').length, 1);
    assert.equal(checkBranch('onion-skin').length, 1);
  });
});

describe('countReviewedLines', () => {
  it('sums added and deleted lines, generated, vendored and binary files aside', () => {
    const numstat = [
      '10\t2\tcrates/core/src/lib.rs',
      '300\t100\tCargo.lock',
      '50\t0\tfrontend/package-lock.json',
      '80\t80\tcrates/server/openapi.json',
      '40\t0\tcrates/compiler/tests/golden/walk.gif.txt',
      '-\t-\tsamples/walk.png',
      '3\t1\tfrontend/projects/app/src/main.ts',
      '',
    ].join('\n');

    assert.equal(countReviewedLines(numstat), 16);
  });
});
