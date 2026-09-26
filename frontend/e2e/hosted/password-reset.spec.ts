import { expectAccessible } from './accessibility';
import { expect, test } from './fixtures';
import { randomPassword } from './identities';
import { lastMessage, linkIn } from './mailpit';
import { AccountPages } from './pages/account-pages';
import { HOSTED_APP_URL } from './stack/ports';

// A password reset by email (support-admin.md, H17): asked from the sign-in page, set from the
// emailed link; the new password signs in, the old one no longer does.
test('a forgotten password is reset from the emailed link', async ({ page, person }) => {
  const account = new AccountPages(page);
  const newPassword = randomPassword();
  await account.signUp(person);
  await account.signOut();

  await test.step('asks for a reset from the sign-in page', async () => {
    await page.goto('/sign-in');
    await expectAccessible(page, 'the sign-in page');
    await page.getByRole('link', { name: 'Forgot your password?' }).click();
    await account.expectPage('Reset your password');
    await account.emailField().fill(person.email);
    await account.button('Send the link').click();
    await expect(page.getByText(/^If an account uses this address/)).toBeVisible();
    await expectAccessible(page, 'the reset page');
  });

  await test.step('sets a new password from the email', async () => {
    const email = await lastMessage(person.email, 'Reset your Life Pixel password');
    await page.goto(linkIn(email, HOSTED_APP_URL).href);
    await account.expectPage('Set a new password');
    await expectAccessible(page, 'the new-password page');
    await account.passwordField('New password').fill(newPassword);
    await account.button('Set the new password').click();
    await expect(page.getByText(/^Your password is set\./)).toBeVisible();
  });

  await test.step('the old password is refused, the new one signs in', async () => {
    await page.goto('/sign-in');
    await account.submitSignIn(person.email, person.password);
    await expect(account.formAlert()).toHaveText(
      'The email address or the password is not correct.',
    );
    await account.signIn(person, newPassword);
  });
});
