import { readFile } from 'node:fs/promises';
import { expect, test } from './fixtures';
import { AccountPages } from './pages/account-pages';
import { HostedEditor } from './pages/hosted-editor';
import { readZip } from './zip';

/** A document's path in the export: laid out like a local library (service.md, H2). */
const DOCUMENT_PATH = /^projects\/[0-9a-f-]{36}\/animations\/[0-9a-f-]{36}\.json$/;

// The data export (support-admin.md, H17): downloaded from the account page, a zip laid out like
// a local library, holding library.json and every document.
test('the data export holds library.json and the documents', async ({ page, person }) => {
  const account = new AccountPages(page);
  const editor = new HostedEditor(page);
  await account.signUp(person);
  for (const title of ['Export one', 'Export two']) {
    await editor.createAnimation(title);
    await editor.draw();
    await editor.save();
    await (title === 'Export one'
      ? editor.saveInNewProject('Archive')
      : editor.saveInProject('Archive'));
    await editor.expectSaved();
  }

  await account.openAccount();
  const [download] = await Promise.all([
    page.waitForEvent('download'),
    page.getByRole('link', { name: 'Download my data' }).click(),
  ]);
  expect(download.suggestedFilename()).toMatch(/^life-pixel-export-\d{4}-\d{2}-\d{2}\.zip$/);
  const entries = readZip(await readFile(await download.path()));
  const names = entries.map((entry) => entry.name);

  expect(names).toContain('library.json');
  expect(names).toContain('account.json');
  const profile = entries.find((entry) => entry.name === 'account.json');
  expect(JSON.parse(profile?.read().toString() ?? '{}')).toMatchObject({ email: person.email });
  const documents = entries.filter((entry) => DOCUMENT_PATH.test(entry.name));
  const titles = documents.map((entry) => JSON.parse(entry.read().toString()).title as string);
  expect(titles.sort()).toEqual(['Export one', 'Export two']);
});
