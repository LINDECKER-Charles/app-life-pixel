import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { needsVerification } from './verification-scope.mjs';

describe('needsVerification', () => {
  it('skips documentation: docs/, and Markdown anywhere else', () => {
    for (const path of [
      'docs/devops.md',
      'docs/v1/diagram.svg',
      'README.md',
      'crates/format/README.md',
      '.github/CONTRIBUTING.md',
    ]) {
      assert.equal(needsVerification(path), false, path);
    }
  });

  it('verifies the Markdown code reads: the legal pages of i18n/', () => {
    assert.equal(needsVerification('i18n/legal/fr/terms.md'), true);
  });

  it('verifies everything else', () => {
    for (const path of [
      'crates/core/src/limits.rs',
      'Cargo.lock',
      'frontend/package.json',
      '.github/workflows/_verify.yml',
      'compose.yaml',
      'design-system/assets/pip.svg',
      'i18n/fr.json',
    ]) {
      assert.equal(needsVerification(path), true, path);
    }
  });
});
