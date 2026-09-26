import { expect, type Locator, type Page } from '@playwright/test';

/** A support request as the form sends it. */
export interface NewRequest {
  readonly category: string;
  readonly message: string;
  readonly screenshot: Buffer;
}

/** The app's support pages (support-admin.md, H9): the new request form, the list, a thread. */
export class SupportPages {
  constructor(readonly page: Page) {}

  async open(): Promise<void> {
    await this.page.goto('/support');
    await expect(
      this.page.getByRole('heading', { name: 'Help and support', level: 1 }),
    ).toBeVisible();
  }

  region(name: string): Locator {
    return this.page.getByRole('region', { name });
  }

  /** Fills "New request" — the category from its select —, attaches the screenshot, and sends. */
  async send(request: NewRequest): Promise<void> {
    const form = this.region('New request');
    await form.locator('ion-select').click();
    await this.page.getByRole('radio', { name: request.category, exact: true }).click();
    await form.getByRole('textbox', { name: 'Message' }).fill(request.message);
    await form.getByLabel('Screenshot').setInputFiles({
      name: 'screenshot.png',
      mimeType: 'image/png',
      buffer: request.screenshot,
    });
    await form.getByRole('button', { name: 'Send the request' }).click();
    await expect(form.getByRole('status')).toHaveText(
      'Your request was sent. We will answer here and by email.',
    );
  }

  /** Opens a request of "Your requests", by its category. */
  async openRequest(category: string): Promise<void> {
    await this.region('Your requests').getByRole('link', { name: category }).click();
    await this.expectThread();
  }

  async expectThread(): Promise<void> {
    await expect(
      this.page.getByRole('heading', { name: 'Support request', level: 1 }),
    ).toBeVisible();
  }

  /** The thread's messages, in their order. */
  messages(): Locator {
    return this.region('Messages').getByRole('listitem');
  }
}
