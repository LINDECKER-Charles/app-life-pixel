import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import {
  checkCatalogueText,
  checkLanguageList,
  checkMessages,
  checkSameKeys,
  formatCatalogue,
} from './catalogue-rules.mjs';

describe('checkCatalogueText', () => {
  it('accepts sorted flat keys, one per line', () => {
    const text = formatCatalogue({ 'common.ok': 'OK', 'app.name': 'Life Pixel' });
    assert.deepEqual(checkCatalogueText('en.json', text), []);
  });

  it('rejects unsorted keys', () => {
    const text = '{\n  "common.ok": "OK",\n  "app.name": "Life Pixel"\n}\n';
    assert.deepEqual(checkCatalogueText('en.json', text), ['en.json: keys are not sorted']);
  });

  it('rejects nested messages and keys outside the pattern', () => {
    const text = JSON.stringify({ 'Common.OK': 'OK', app: { name: 'Life Pixel' } }, null, 2);
    assert.deepEqual(checkCatalogueText('en.json', text), [
      'en.json: not flat: app',
      'en.json: invalid key: Common.OK',
      'en.json: invalid key: app',
    ]);
  });

  it('rejects a repeated key and a layout other than one key per line', () => {
    const repeated = '{\n  "app.name": "A",\n  "app.name": "B"\n}\n';
    const inline = '{ "app.name": "A" }\n';
    assert.equal(checkCatalogueText('en.json', repeated).length, 1);
    assert.equal(checkCatalogueText('en.json', inline).length, 1);
  });

  it('rejects text that is not JSON', () => {
    assert.match(checkCatalogueText('fr.json', '{')[0], /^fr\.json: not valid JSON/);
  });
});

describe('checkSameKeys', () => {
  it('names the missing and the extra keys', () => {
    const source = { 'app.name': 'Life Pixel', 'common.ok': 'OK' };
    const catalogue = { 'app.name': 'Life Pixel', 'common.extra': 'En plus' };
    assert.deepEqual(checkSameKeys('fr.json', catalogue, source), [
      'fr.json: missing common.ok',
      'fr.json: common.extra is not in the source catalogue',
    ]);
  });
});

describe('checkMessages', () => {
  it('accepts ICU plurals with the categories of the language', () => {
    const catalogue = { 'editor.frames': '{count, plural, one {# image} other {# images}}' };
    assert.deepEqual(checkMessages('fr.json', 'fr', catalogue), []);
  });

  it('rejects a message that does not parse', () => {
    const problems = checkMessages('en.json', 'en', { 'editor.frames': '{count, plural, one {' });
    assert.match(problems[0], /^en\.json: editor\.frames does not parse/);
  });

  it('rejects a plural category the language does not have', () => {
    const catalogue = { 'editor.frames': '{count, plural, few {# frames} other {# frames}}' };
    assert.equal(checkMessages('en.json', 'en', catalogue).length, 1);
  });

  it('allows only simple arguments in emails', () => {
    const catalogue = {
      'email.verify.body': 'Open {link}',
      'email.verify.count': '{count, plural, one {# day} other {# days}}',
    };
    assert.deepEqual(checkMessages('en.json', 'en', catalogue), [
      'en.json: email.verify.count uses more than simple {name} arguments',
    ]);
  });
});

describe('checkLanguageList', () => {
  it('accepts a list of exactly the catalogues', () => {
    const languages = [
      { code: 'en', name: 'English' },
      { code: 'fr', name: 'Français' },
    ];
    assert.deepEqual(checkLanguageList(languages, ['fr', 'en']), []);
  });

  it('names missing, unknown, repeated and nameless languages', () => {
    const languages = [
      { code: 'en', name: 'English' },
      { code: 'en', name: 'English' },
      { code: 'de' },
    ];
    assert.deepEqual(checkLanguageList(languages, ['en', 'fr']), [
      'languages.json: needs a code and a name: {"code":"de"}',
      'languages.json: en is listed twice',
      'languages.json: misses fr, whose catalogue exists',
      'languages.json: lists de, which has no catalogue',
    ]);
  });
});
