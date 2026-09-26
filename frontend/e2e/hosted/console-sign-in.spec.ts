import { expectAccessible } from './accessibility';
import { expect, test } from './fixtures';
import { sharedAdmin } from './stack/admin';
import { ADMIN_CONSOLE_URL } from './stack/ports';
import { authenticator, freshCode } from './totp';

// The console's sign-in (support-admin.md, H11 and H12), which the other journeys skip with the
// session the global setup keeps: an address, a password and a TOTP code of a step not used yet.
test('an admin signs in to the console with a TOTP code', async ({ browser }) => {
  const admin = sharedAdmin();
  const context = await browser.newContext({ baseURL: ADMIN_CONSOLE_URL, locale: 'en-US' });
  const page = await context.newPage();

  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Sign in to the admin console' })).toBeVisible();
  await expectAccessible(page, 'the console sign-in page');
  await page.getByRole('textbox', { name: 'Email address' }).fill(admin.email);
  await page.getByLabel('Password', { exact: true }).fill(admin.password);
  const { code } = await freshCode(authenticator(admin.otpauthUri), admin.usedStep);
  await page.getByRole('textbox', { name: 'Authenticator code' }).fill(code);
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();

  await expect(page.getByRole('heading', { level: 1 })).not.toHaveText(
    'Sign in to the admin console',
  );
  await expect(page.getByRole('button', { name: 'Sign out' })).toBeVisible();
  await expectAccessible(page, 'the console overview');
  await context.close();
});
