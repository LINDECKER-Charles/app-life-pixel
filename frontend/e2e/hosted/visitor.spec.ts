import { expectAccessible } from './accessibility';
import { expect, test } from './fixtures';
import { lastMessage, linkIn } from './mailpit';
import { AccountPages } from './pages/account-pages';
import { HostedEditor } from './pages/hosted-editor';
import { LibraryPage } from './pages/library-page';
import { HOSTED_APP_URL } from './stack/ports';

// M3's first journey (support-admin.md, H17): a visitor's work survives signing up, lands in a
// new project, and the address is verified from the email.
test('a visitor draws, signs up to save, and verifies the address', async ({ page, person }) => {
  const editor = new HostedEditor(page);
  const account = new AccountPages(page);

  await test.step('a visitor draws, then saves', async () => {
    await editor.createAnimation('First steps');
    await expectAccessible(page, 'the editor, signed out');
    await editor.draw();
    await editor.save();
    await expect(editor.saveDialog('Sign in to save')).toBeVisible();
    await expectAccessible(page, 'the sign-in-to-save dialog');
  });

  await test.step('signs up from the save dialog', async () => {
    await editor.button('Create an account').click();
    await account.expectPage('Create an account');
    await expectAccessible(page, 'the sign-up page');
    await account.submitSignUp(person);
  });

  await test.step('comes back to the editor, and saves into a new project', async () => {
    await expect(editor.saveDialog('Save the animation')).toBeVisible();
    await expectAccessible(page, 'the project dialog');
    await editor.saveInNewProject('Sketches');
    await editor.expectSaved();
    await expect(editor.titleField()).toHaveValue('First steps');
  });

  await test.step('verifies the address from the email', async () => {
    const email = await lastMessage(person.email, 'Verify your Life Pixel email address');
    await page.goto(linkIn(email, HOSTED_APP_URL).href);
    await expect(page.getByText('Your address is verified.')).toBeVisible();
    await expectAccessible(page, 'the verification page');
    await page.getByRole('link', { name: 'Go to your account' }).click();
    await account.expectPage('Account');
    await expect(page.getByText('This address is verified.')).toBeVisible();
    await expectAccessible(page, 'the account page');
  });

  await test.step('finds the animation in the new project of the library', async () => {
    const library = new LibraryPage(page);
    await library.open();
    await expect(library.project('Sketches')).toBeVisible();
    await expect(library.animation('First steps')).toBeVisible();
    await expectAccessible(page, 'the library');
  });
});
