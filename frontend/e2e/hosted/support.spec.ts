import { expectAccessible } from './accessibility';
import { expect, test } from './fixtures';
import { lastMessage, linkIn } from './mailpit';
import { AccountPages } from './pages/account-pages';
import { AdminConsole } from './pages/admin-console';
import { SupportPages } from './pages/support-pages';
import { HOSTED_APP_URL } from './stack/ports';

const QUESTION = 'The export dialog closes when I pick APNG. The screenshot shows where.';
const ANSWER = 'Thank you: this is fixed in the next version. Pick GIF meanwhile.';

// A support request with a screenshot, answered from the admin console (support-admin.md, H17):
// the reply is seen in the app, and received by email with a link to the thread.
test('a support request is answered from the admin console', async ({
  page,
  person,
  consolePage,
}) => {
  const support = new SupportPages(page);
  const admin = new AdminConsole(consolePage);
  await new AccountPages(page).signUp(person);

  await test.step('the person sends a request with a screenshot', async () => {
    await support.open();
    await expectAccessible(page, 'the support page');
    await support.send({ category: 'Bug', message: QUESTION, screenshot: await page.screenshot() });
    await support.openRequest('Bug');
    await expect(page.getByText('A screenshot came with this request.')).toBeVisible();
    await expect(support.messages()).toHaveCount(1);
    await expectAccessible(page, 'the support thread');
  });

  await test.step('an admin finds it in the queue, with its screenshot', async () => {
    await admin.openRequestOf(person.email);
    await expect(consolePage.getByText(QUESTION)).toBeVisible();
    await admin.button('Show the screenshot').click();
    const screenshot = consolePage.getByRole('img', {
      name: 'The screenshot the user sent with the request',
    });
    await expect(screenshot).toBeVisible();
    await expect
      .poll(() => screenshot.evaluate((image: HTMLImageElement) => image.naturalWidth))
      .toBeGreaterThan(0);
    await expectAccessible(consolePage, 'the console support thread');
  });

  await test.step('the admin replies', async () => {
    await admin.reply(ANSWER);
    await expect(consolePage.getByText(ANSWER)).toBeVisible();
  });

  await test.step('the person sees the reply in the app', async () => {
    await page.reload();
    await expect(support.messages()).toHaveCount(2);
    await expect(support.messages().nth(1)).toContainText('Life Pixel support');
    await expect(support.messages().nth(1)).toContainText(ANSWER);
    await expect(page.getByText('Status: Waiting for your reply')).toBeVisible();
  });

  await test.step('and receives it by email, linking to the thread', async () => {
    const email = await lastMessage(person.email, 'Life Pixel support answered your request');
    const link = linkIn(email, HOSTED_APP_URL);
    expect(link.pathname).toBe(new URL(page.url()).pathname);
    await page.goto(link.href);
    await expect(support.messages().nth(1)).toContainText(ANSWER);
  });
});
