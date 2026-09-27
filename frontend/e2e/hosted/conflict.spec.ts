import { expectAccessible } from './accessibility';
import { expect, test } from './fixtures';
import { AccountPages } from './pages/account-pages';
import { HostedEditor } from './pages/hosted-editor';
import { LibraryPage } from './pages/library-page';

// Two pages saving one animation (support-admin.md, H17): the second learns that the first saved
// since it opened the animation, and keeps its work as a copy.
test('the second of two pages saving one animation saves a copy', async ({ page, person }) => {
  const first = new HostedEditor(page);
  await new AccountPages(page).signUp(person);
  await first.createAnimation('Shared');
  await first.draw();
  await first.save();
  await first.saveInNewProject('Together');
  await first.expectSaved();
  const id = first.savedId();

  const second = new HostedEditor(await page.context().newPage());
  await test.step('a second page opens the same animation', async () => {
    await second.page.goto(`/editor/${id}`);
    await expect(second.titleField()).toHaveValue('Shared');
  });

  await test.step('the first page changes it and saves', async () => {
    await first.draw();
    await first.save();
    await first.expectSaved();
  });

  await test.step('the second page changes it, saves, and learns of the conflict', async () => {
    await second.draw();
    await second.save();
    await expect(second.saveDialog('Changed elsewhere')).toBeVisible();
    await expectAccessible(second.page, 'the conflict dialog');
  });

  await test.step('it chooses "Save a copy", and follows the copy', async () => {
    await second.button('Save a copy').click();
    await second.expectSaved();
    expect(second.savedId()).not.toBe(id);
    await expect(second.titleField()).toHaveValue('Copy of Shared');
  });

  await test.step('the library holds the original and the copy', async () => {
    const library = new LibraryPage(second.page);
    await library.open();
    await library.expectTitles(['Copy of Shared', 'Shared']);
  });
});
