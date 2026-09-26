import { test as base, type BrowserContext, type Dialog, type Page } from '@playwright/test';
import { newPerson, type Person } from './identities';
import { ADMIN_STORAGE_STATE } from './stack/admin';
import { ADMIN_CONSOLE_URL } from './stack/ports';

interface HostedFixtures {
  /** The journey's person: a new address, and a client address of their own. */
  readonly person: Person;
  /** A page of the admin console, signed in as the global setup's admin. */
  readonly consolePage: Page;
}

/**
 * Leaving a page that holds unsaved work asks first (D37): the journeys leave on purpose, so they
 * accept; any other dialog is dismissed, as Playwright does by default.
 */
function answer(dialog: Dialog): Promise<void> {
  return dialog.type() === 'beforeunload' ? dialog.accept() : dialog.dismiss();
}

async function asPerson(context: BrowserContext, person: Person): Promise<void> {
  await context.setExtraHTTPHeaders({ 'X-Forwarded-For': person.clientAddress });
  context.on('page', (page) => page.on('dialog', answer));
}

/** The hosted journeys' test: each gets its person, and the admin console when it asks. */
export const test = base.extend<HostedFixtures>({
  // eslint-disable-next-line no-empty-pattern
  person: async ({}, use) => {
    await use(newPerson('user'));
  },
  context: async ({ context, person }, use) => {
    await asPerson(context, person);
    await use(context);
  },
  consolePage: async ({ browser }, use) => {
    const context = await browser.newContext({
      baseURL: ADMIN_CONSOLE_URL,
      storageState: ADMIN_STORAGE_STATE,
      locale: 'en-US',
    });
    await use(await context.newPage());
    await context.close();
  },
});

export { expect } from '@playwright/test';
