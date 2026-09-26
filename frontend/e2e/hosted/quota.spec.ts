import { expectAccessible } from './accessibility';
import { fillerDocument } from './filler';
import { expect, test } from './fixtures';
import { LibraryApi } from './library-api';
import { AccountPages } from './pages/account-pages';
import { HostedEditor } from './pages/hosted-editor';
import { LibraryPage } from './pages/library-page';
import { STORAGE_QUOTA_BYTES } from './stack/hosted-stack';

const PROJECT = 'Full';
const FILLER = 'Filler';

// The quota reached and explained (support-admin.md, H17), under the 200,000 bytes the global
// setup gives the free plan: the save that would exceed it says why and where to free space.
test('a save beyond the quota is explained, and works once space is freed', async ({
  page,
  person,
}) => {
  const editor = new HostedEditor(page);
  const api = new LibraryApi(page);
  const account = new AccountPages(page);
  await account.signUp(person);
  await expect(page.getByText('0 of 200,000 bytes used.')).toBeVisible();

  const firstSave = await test.step('a first animation is saved', async () => {
    await editor.createAnimation('Small');
    await editor.draw();
    await editor.save();
    await editor.saveInNewProject(PROJECT);
    await editor.expectSaved();
    return api.usedBytes();
  });

  await test.step('the library fills up to less than one more such animation', async () => {
    const room = Math.floor(firstSave / 2);
    const filler = fillerDocument(FILLER, STORAGE_QUOTA_BYTES - firstSave - room);
    await api.createAnimation(await api.projectId(PROJECT), filler);
    const left = STORAGE_QUOTA_BYTES - (await api.usedBytes());
    expect(left).toBeGreaterThanOrEqual(0);
    expect(left).toBeLessThan(firstSave);
  });

  await test.step('saving a second animation is refused, and explained', async () => {
    await editor.newAnimation('One too many');
    await editor.draw();
    await editor.save();
    await editor.saveInProject(PROJECT);
    const dialog = editor.saveDialog('Storage full');
    await expect(dialog).toBeVisible();
    await expect(
      page.getByText(/Your storage is full: [\d,]+ of 200,000 bytes are used\./),
    ).toBeVisible();
    await expectAccessible(page, 'the storage-full dialog');
  });

  await test.step('the library, opened from the dialog, frees space', async () => {
    await page.getByRole('link', { name: 'Open the library' }).click();
    const library = new LibraryPage(page);
    await expect(library.animation(FILLER)).toBeVisible();
    await library.delete(FILLER);
    await expect(library.animation(FILLER)).toBeHidden();
  });

  await test.step('back in the editor, the work is still there, and saves', async () => {
    await page.getByRole('link', { name: 'Editor', exact: true }).click();
    await editor.settled();
    await expect(editor.titleField()).toHaveValue('One too many');
    await editor.save();
    await editor.saveInProject(PROJECT);
    await editor.expectSaved();
  });
});
