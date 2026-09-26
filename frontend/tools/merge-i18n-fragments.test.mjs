import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { afterEach, beforeEach, describe, it } from 'node:test';
import { fileURLToPath } from 'node:url';
import { checkCatalogueText, formatCatalogue } from './i18n/catalogue-rules.mjs';
import { mergeFragments } from './merge-i18n-fragments.mjs';

const TOOL = fileURLToPath(new URL('merge-i18n-fragments.mjs', import.meta.url));

let repository;

function write(path, text) {
  const file = join(repository, path);
  mkdirSync(join(file, '..'), { recursive: true });
  writeFileSync(file, text);
}

function writeJson(path, value) {
  write(path, `${JSON.stringify(value, null, 2)}\n`);
}

function catalogue(code) {
  return JSON.parse(readFileSync(join(repository, 'i18n', `${code}.json`), 'utf8'));
}

function catalogueText(code) {
  return readFileSync(join(repository, 'i18n', `${code}.json`), 'utf8');
}

beforeEach(() => {
  repository = mkdtempSync(join(tmpdir(), 'lp-i18n-fragments-'));
  write('i18n/en.json', formatCatalogue({ 'common.ok': 'OK', 'editor.old': 'Old' }));
  write('i18n/fr.json', formatCatalogue({ 'common.ok': 'OK', 'editor.old': 'Ancien' }));
  writeJson('i18n/languages.json', [{ code: 'en', name: 'English' }]);
  write('i18n-pending/README.md', '# Pending translations\n');
});

afterEach(() => rmSync(repository, { recursive: true, force: true }));

describe('mergeFragments', () => {
  it('adds and replaces keys, writes sorted catalogues and deletes the fragments', () => {
    writeJson('i18n-pending/lot3/en.json', { 'zoom.in': 'Zoom in', 'common.ok': 'Fine' });
    writeJson('i18n-pending/lot3/fr.json', { 'zoom.in': 'Zoomer', 'common.ok': 'D’accord' });

    const result = mergeFragments(repository);

    assert.deepEqual(result, { lots: ['lot3'], warnings: [] });
    assert.deepEqual(catalogue('en'), {
      'common.ok': 'Fine',
      'editor.old': 'Old',
      'zoom.in': 'Zoom in',
    });
    assert.equal(catalogue('fr')['zoom.in'], 'Zoomer');
    assert.deepEqual(checkCatalogueText('en.json', catalogueText('en')), []);
    assert.deepEqual(checkCatalogueText('fr.json', catalogueText('fr')), []);
    assert.equal(existsSync(join(repository, 'i18n-pending/lot3')), false);
    assert.equal(existsSync(join(repository, 'i18n-pending/README.md')), true);
  });

  it('drops the keys of removed.txt from every catalogue, skipping blanks and comments', () => {
    write('i18n-pending/lot5/removed.txt', '# gone with the old timeline\n\neditor.old\r\n');

    mergeFragments(repository);

    assert.deepEqual(catalogue('en'), { 'common.ok': 'OK' });
    assert.deepEqual(catalogue('fr'), { 'common.ok': 'OK' });
    assert.equal(existsSync(join(repository, 'i18n-pending/lot5')), false);
  });

  it('keeps the fragments with keep', () => {
    writeJson('i18n-pending/lot4/en.json', { 'tools.zoom': 'Zoom' });
    writeJson('i18n-pending/lot4/fr.json', { 'tools.zoom': 'Zoom' });

    mergeFragments(repository, { keep: true });

    assert.equal(catalogue('en')['tools.zoom'], 'Zoom');
    assert.equal(existsSync(join(repository, 'i18n-pending/lot4/en.json')), true);
    assert.equal(existsSync(join(repository, 'i18n-pending/lot4/fr.json')), true);
  });

  it('changes nothing without fragments', () => {
    const unsorted = '{\n  "zoom.in": "Zoom in",\n  "common.ok": "OK"\n}\n';
    write('i18n/en.json', unsorted);
    mkdirSync(join(repository, 'i18n-pending/lot7'));

    assert.deepEqual(mergeFragments(repository), { lots: [], warnings: [] });
    assert.equal(catalogueText('en'), unsorted);
  });

  it('applies lots in name order and reports what deserves a look', () => {
    writeJson('i18n-pending/lot3/en.json', { 'common.save': 'Save' });
    writeJson('i18n-pending/lot3/fr.json', { 'common.save': 'Enregistrer' });
    writeJson('i18n-pending/lot7/en.json', { 'common.save': 'Save now', 'export.copy': 'Copy' });
    writeJson('i18n-pending/lot7/fr.json', { 'common.save': 'Enregistrer' });

    const { warnings } = mergeFragments(repository);

    assert.equal(catalogue('en')['common.save'], 'Save now');
    assert.deepEqual(warnings, [
      'en.json: common.save set by lot3, replaced by lot7',
      'lot7: export.copy is missing from fr',
    ]);
  });

  it('rejects a nested fragment and a language without a catalogue, writing nothing', () => {
    writeJson('i18n-pending/lot8/en.json', { common: { ok: 'OK' } });
    assert.throws(() => mergeFragments(repository), /lot8\/en\.json: not a flat object/);

    rmSync(join(repository, 'i18n-pending/lot8'), { recursive: true });
    writeJson('i18n-pending/lot8/de.json', { 'common.ok': 'OK' });
    assert.throws(() => mergeFragments(repository), /there is no i18n\/de\.json/);
    assert.deepEqual(catalogue('en'), { 'common.ok': 'OK', 'editor.old': 'Old' });
  });
});

describe('the command', () => {
  it('rejects an unknown argument', () => {
    const { status, stderr } = spawnSync(process.execPath, [TOOL, '--force'], { encoding: 'utf8' });
    assert.equal(status, 2);
    assert.match(stderr, /usage: .*\[--keep\]/);
  });
});
