import assert from 'node:assert/strict';
import { it } from 'node:test';
import { findUnusedKeys } from './unused-keys.mjs';

it('reports keys no source names, except those keyed at run time', () => {
  const keys = ['app.name', 'common.ok', 'errors.quota.storage_exceeded', 'email.verify.subject'];
  const sources = [`<h1>{{ 'app.name' | transloco }}</h1>`];
  assert.deepEqual(findUnusedKeys(keys, sources), ['common.ok']);
});
