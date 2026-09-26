import assert from 'node:assert/strict';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { afterEach, beforeEach, describe, it } from 'node:test';
import { fileURLToPath } from 'node:url';
import { copyDesignAssets, DESIGN_ASSETS } from './copy-design-assets.mjs';

const REAL_SOURCE = fileURLToPath(new URL('../../design-system/assets/', import.meta.url));
const GITIGNORE = fileURLToPath(new URL('../.gitignore', import.meta.url));

let workspace;
let source;
let target;

function writeAsset(root, asset, text) {
  const file = join(root, asset);
  mkdirSync(dirname(file), { recursive: true });
  writeFileSync(file, text);
}

beforeEach(() => {
  workspace = mkdtempSync(join(tmpdir(), 'lp-design-assets-'));
  source = join(workspace, 'assets');
  target = join(workspace, 'public', 'design-system');
  for (const asset of DESIGN_ASSETS) writeAsset(source, asset, `content of ${asset}`);
});

afterEach(() => rmSync(workspace, { recursive: true, force: true }));

describe('copyDesignAssets', () => {
  it('finds every asset the app uses in design-system/assets/', () => {
    const missing = DESIGN_ASSETS.filter((asset) => !existsSync(join(REAL_SOURCE, asset)));

    assert.deepEqual(missing, []);
  });

  it('writes each asset to the target, byte for byte', () => {
    const written = copyDesignAssets(source, target);

    assert.equal(written.length, DESIGN_ASSETS.length);
    for (const asset of DESIGN_ASSETS) {
      assert.equal(readFileSync(join(target, asset), 'utf8'), `content of ${asset}`);
    }
  });

  it('ships the font with its licence', () => {
    copyDesignAssets(source, target);

    assert.ok(existsSync(join(target, 'fonts', 'nunito-variable.ttf')));
    assert.ok(existsSync(join(target, 'fonts', 'OFL.txt')));
  });

  it('removes what an earlier copy left behind', () => {
    writeAsset(target, 'old.svg', 'stale');

    copyDesignAssets(source, target);

    assert.equal(existsSync(join(target, 'old.svg')), false);
  });

  it('fails without touching the target when a source is missing', () => {
    writeAsset(target, 'mark.svg', 'previous copy');
    rmSync(join(source, 'pip.svg'));

    assert.throws(() => copyDesignAssets(source, target), /missing in .*pip\.svg/);
    assert.equal(readFileSync(join(target, 'mark.svg'), 'utf8'), 'previous copy');
  });

  it('keeps the copy out of Git', () => {
    const ignored = readFileSync(GITIGNORE, 'utf8').split(/\r?\n/);

    assert.ok(ignored.includes('/projects/app/public/design-system/'));
  });
});
