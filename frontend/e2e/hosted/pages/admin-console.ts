import { expect, type Locator, type Page } from '@playwright/test';

/** A user action of the console, as its button, its confirmation and its notice name it. */
export interface UserAction {
  readonly button: string;
  readonly dialog: string;
  readonly confirm: RegExp;
  readonly done: string;
}

export const SUSPEND: UserAction = {
  button: 'Suspend',
  dialog: 'Suspend this account?',
  confirm: /^Suspend in /,
  done: 'The account is suspended.',
};

export const REACTIVATE: UserAction = {
  button: 'Reactivate',
  dialog: 'Reactivate this account?',
  confirm: /^Reactivate in /,
  done: 'The account is reactivated.',
};

/** The admin console (support-admin.md, H12): users and support, as an admin finds them. */
export class AdminConsole {
  constructor(readonly page: Page) {}

  heading(name: string): Locator {
    return this.page.getByRole('heading', { name, level: 1, exact: true });
  }

  button(name: string): Locator {
    return this.page.getByRole('button', { name, exact: true });
  }

  /** The notice a finished action shows, among the page's other statuses. */
  notice(text: string): Locator {
    return this.page.getByRole('status').filter({ hasText: text });
  }

  /** Finds the account of `email` from the users page, and opens it. */
  async openUser(email: string): Promise<void> {
    await this.page.goto('/users');
    await expect(this.heading('Users')).toBeVisible();
    await this.page.getByRole('searchbox', { name: 'Email address or id' }).fill(email);
    await this.button('Search').click();
    await this.page.getByRole('link', { name: email, exact: true }).click();
    await expect(this.heading(email)).toBeVisible();
  }

  /** Asks for `action` on the open account; the confirmation shows, awaiting a reason. */
  async ask(action: UserAction): Promise<Locator> {
    await this.button(action.button).click();
    const dialog = this.page.getByRole('dialog', { name: action.dialog });
    await expect(dialog).toBeVisible();
    return dialog;
  }

  /** Confirms `action` in its dialog with `reason`, and waits for its notice. */
  async confirm(dialog: Locator, action: UserAction, reason: string): Promise<void> {
    await dialog.getByRole('textbox', { name: 'Reason' }).fill(reason);
    await dialog.getByRole('button', { name: action.confirm }).click();
    await expect(dialog).toBeHidden();
    await expect(this.notice(action.done)).toBeVisible();
  }

  /** Opens the support request of `email` from the queue. */
  async openRequestOf(email: string): Promise<void> {
    await this.page.goto('/support');
    await expect(this.heading('Support')).toBeVisible();
    const row = this.page.getByRole('row').filter({ hasText: email });
    await row.getByRole('link').click();
    await expect(this.page.getByRole('region', { name: 'Messages' })).toBeVisible();
  }

  /** Sends `body` as a reply, emailed to the user. */
  async reply(body: string): Promise<void> {
    await this.page.getByRole('radio', { name: /^Reply, emailed to / }).check();
    await this.page.getByRole('textbox', { name: 'Reply', exact: true }).fill(body);
    await this.button('Send the reply').click();
    await expect(this.notice('The reply is sent.')).toBeVisible();
  }
}
