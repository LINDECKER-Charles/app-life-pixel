import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { CODEQL_LANGUAGES, codeqlLanguagesFor, needsVerification } from './verification-scope.mjs';

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

describe('codeqlLanguagesFor', () => {
  it('names each language whose sources changed, once, in a stable order', () => {
    const paths = [
      'frontend/projects/app/src/main.ts',
      'crates/core/src/limits.rs',
      '.github/workflows/ci.yml',
      'crates/core/src/model.rs',
    ];

    assert.deepEqual(codeqlLanguagesFor(paths), CODEQL_LANGUAGES);
  });

  it('counts the manifests of Rust and the templates of the front end as sources', () => {
    assert.deepEqual(codeqlLanguagesFor(['crates/server/Cargo.toml']), ['rust']);
    assert.deepEqual(codeqlLanguagesFor(['Cargo.lock']), ['rust']);
    assert.deepEqual(codeqlLanguagesFor(['frontend/projects/app/src/app/app.html']), [
      'javascript-typescript',
    ]);
  });

  it('names none for a change no analysis reads', () => {
    assert.deepEqual(codeqlLanguagesFor(['docs/devops.md', 'i18n/fr.json', 'compose.yaml']), []);
  });
});
