import { expectAccessible } from './accessibility';
import { expect, test } from './fixtures';
import { AccountPages } from './pages/account-pages';
import { HostedEditor } from './pages/hosted-editor';
import { LibraryPage } from './pages/library-page';

// The library's journey (support-admin.md, H17): search, rename, duplicate, delete.
test('the library searches, renames, duplicates and deletes', async ({ page, person }) => {
  const editor = new HostedEditor(page);
  const library = new LibraryPage(page);
  await new AccountPages(page).signUp(person);

  await test.step('two animations are saved into one project', async () => {
    await editor.createAnimation('Walk cycle');
    await editor.draw();
    await editor.save();
    await editor.saveInNewProject('Hero');
    await editor.expectSaved();
    await editor.createAnimation('Idle breath');
    await editor.draw();
    await editor.save();
    await editor.saveInProject('Hero');
    await editor.expectSaved();
  });

  await test.step('the library lists both, and a search finds one', async () => {
    await library.open();
    await expect(library.project('Hero')).toBeVisible();
    await library.expectTitles(['Idle breath', 'Walk cycle']);
    await expectAccessible(page, 'the library');
    await library.search('walk');
    await library.expectTitles(['Walk cycle']);
  });

  await test.step('renames an animation', async () => {
    await library.rename('Walk cycle', 'Run cycle');
    await expect(library.animation('Run cycle')).toBeVisible();
    await expect(library.animation('Walk cycle')).toBeHidden();
  });

  await test.step('duplicates it', async () => {
    await library.duplicate('Run cycle');
    await expect(library.animation('Copy of Run cycle')).toBeVisible();
  });

  await test.step('deletes the copy, after a confirmation', async () => {
    await library.animations.getByRole('button', { name: 'Delete Copy of Run cycle' }).click();
    await expect(library.alert('Delete this animation?')).toBeVisible();
    await expectAccessible(page, 'the deletion confirmation');
    await library.alert('Delete this animation?').getByRole('button', { name: 'Cancel' }).click();
    await library.delete('Copy of Run cycle');
    await expect(library.animation('Copy of Run cycle')).toBeHidden();
  });

  await test.step('the library, reloaded, keeps the changes', async () => {
    await library.open();
    await library.expectTitles(['Idle breath', 'Run cycle']);
    await library.animation('Run cycle').click();
    await expect(editor.titleField()).toHaveValue('Run cycle');
  });
});
