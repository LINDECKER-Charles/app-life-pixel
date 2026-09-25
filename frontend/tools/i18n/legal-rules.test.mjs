import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { checkLegalPages, legalPlaceholders } from './legal-rules.mjs';

describe('legalPlaceholders', () => {
  it('lists the distinct placeholders of a text, sorted', () => {
    assert.deepEqual(legalPlaceholders('{{publisher}}, {{address}}, {{publisher}} again'), [
      'address',
      'publisher',
    ]);
  });

  it('returns an empty list when there is no placeholder', () => {
    assert.deepEqual(legalPlaceholders('Plain text.'), []);
  });
});

describe('checkLegalPages', () => {
  function pages(overrides = {}) {
    return {
      en: {
        terms: '{{publisher}}',
        privacy: '{{publisher}}, {{address}}',
        notice: '{{publisher}}, {{director}}',
        ...overrides.en,
      },
      fr: {
        terms: '{{publisher}}',
        privacy: '{{publisher}}, {{address}}',
        notice: '{{publisher}}, {{director}}',
        ...overrides.fr,
      },
    };
  }

  it('accepts every language holding the three pages with the same placeholders', () => {
    assert.deepEqual(checkLegalPages(pages()), []);
  });

  it('names a missing page', () => {
    assert.deepEqual(checkLegalPages(pages({ fr: { notice: undefined } })), [
      'legal/fr/notice.md: missing',
    ]);
  });

  it('names a page whose placeholders differ from the reference language', () => {
    assert.deepEqual(checkLegalPages(pages({ fr: { privacy: '{{publisher}}, {{contact}}' } })), [
      'legal/fr/privacy.md: placeholders [{{contact}}, {{publisher}}] differ from ' +
        "legal/en/privacy.md's [{{address}}, {{publisher}}]",
    ]);
  });
});
