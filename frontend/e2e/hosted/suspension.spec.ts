import { expectAccessible } from './accessibility';
import { expect, test } from './fixtures';
import { AccountPages } from './pages/account-pages';
import { AdminConsole, REACTIVATE, SUSPEND } from './pages/admin-console';

// A user suspended then reactivated from the admin console (support-admin.md, H17): suspending
// ends the sessions and refuses sign-ins until the account is reactivated.
test('a user suspended from the console is reactivated', async ({ page, person, consolePage }) => {
  const account = new AccountPages(page);
  const admin = new AdminConsole(consolePage);
  await account.signUp(person);

  await test.step('an admin finds the account', async () => {
    await admin.openUser(person.email);
    await expectAccessible(consolePage, 'the console user page');
  });

  await test.step('suspends it, with a reason', async () => {
    const dialog = await admin.ask(SUSPEND);
    await expect(dialog).toContainText(person.email);
    await expectAccessible(consolePage, 'the suspension confirmation');
    await admin.confirm(dialog, SUSPEND, 'Spam reported by three people.');
    await expect(consolePage.getByText('Suspended', { exact: true })).toBeVisible();
  });

  await test.step('the person is signed out, and cannot sign in', async () => {
    await page.goto('/account');
    await expect(account.heading('Account')).toBeHidden();
    await page.goto('/sign-in');
    await account.submitSignIn(person.email, person.password);
    await expect(account.formAlert()).toHaveText('This account is suspended.');
  });

  await test.step('the admin reactivates it, and the person signs in again', async () => {
    const dialog = await admin.ask(REACTIVATE);
    await admin.confirm(dialog, REACTIVATE, 'The reports were mistaken.');
    await expect(consolePage.getByText('Active', { exact: true })).toBeVisible();
    await account.signIn(person);
  });
});
