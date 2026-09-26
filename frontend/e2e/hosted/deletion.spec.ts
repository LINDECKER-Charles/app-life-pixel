import { expectAccessible } from './accessibility';
import { expect, test } from './fixtures';
import { AccountPages } from './pages/account-pages';
import { HostedEditor } from './pages/hosted-editor';

// An account deleted (support-admin.md, H17), with its library, from the account page: the
// person is signed out, and signing in fails as for an address nobody uses.
test('an account deleted can no longer sign in', async ({ page, person }) => {
  const account = new AccountPages(page);
  const editor = new HostedEditor(page);
  await account.signUp(person);
  await editor.createAnimation('Farewell');
  await editor.draw();
  await editor.save();
  await editor.saveInNewProject('Last project');
  await editor.expectSaved();

  await test.step('deletes the account, with its password and its address typed', async () => {
    await account.openAccount();
    await account.button('Delete my account').click();
    const confirmation = account.button('Delete my account permanently');
    await expect(confirmation).toBeDisabled();
    await expectAccessible(page, 'the account deletion form');
    await account.passwordField('Password').fill(person.password);
    await page.getByRole('textbox', { name: `Type ${person.email} to confirm` }).fill(person.email);
    await confirmation.click();
    await expect(page).toHaveURL(/\/editor$/);
  });

  await test.step('signing in fails', async () => {
    await page.goto('/sign-in');
    await account.submitSignIn(person.email, person.password);
    await expect(account.formAlert()).toHaveText(
      'The email address or the password is not correct.',
    );
    await page.goto('/account');
    await expect(account.heading('Account')).toBeHidden();
  });
});
