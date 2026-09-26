import { expect, type Locator, type Page } from '@playwright/test';
import type { Person } from '../identities';
import { expectSettled } from './outlet';

/**
 * The account screens of the app (accounts.md, H7) — sign up, sign in, reset, account — as a
 * person finds them: by role and accessible name.
 */
export class AccountPages {
  constructor(readonly page: Page) {}

  heading(name: string): Locator {
    return this.page.getByRole('heading', { name, level: 1, exact: true });
  }

  /** The page titled `name` is shown, the transition to it over. */
  async expectPage(name: string): Promise<void> {
    await expect(this.heading(name)).toBeVisible();
    await expectSettled(this.page);
  }

  /** A button of the page's content, not of the header's account menu. */
  button(name: string): Locator {
    return this.page.getByRole('main').getByRole('button', { name, exact: true });
  }

  emailField(): Locator {
    return this.page.getByRole('main').getByRole('textbox', { name: 'Email address' });
  }

  /** A password field, by its label; its show/hide toggle is a button, not a field. */
  passwordField(label: string): Locator {
    return this.page.getByRole('main').getByLabel(label, { exact: true });
  }

  /** Fills the sign-up form the page shows, and creates the account. */
  async submitSignUp(person: Person): Promise<void> {
    await this.expectPage('Create an account');
    await this.emailField().fill(person.email);
    await this.passwordField('Password').fill(person.password);
    await this.button('Create my account').click();
  }

  /** Creates the account from `/sign-up`, which then opens the account page. */
  async signUp(person: Person): Promise<void> {
    await this.page.goto('/sign-up');
    await this.submitSignUp(person);
    await this.expectPage('Account');
  }

  /** Fills the sign-in form the page shows, with `password`, and signs in. */
  async submitSignIn(email: string, password: string): Promise<void> {
    await this.expectPage('Sign in');
    await this.emailField().fill(email);
    await this.passwordField('Password').fill(password);
    await this.button('Sign in').click();
  }

  /** Signs in from `/sign-in`, which then opens the account page. */
  async signIn(person: Person, password = person.password): Promise<void> {
    await this.page.goto('/sign-in');
    await this.submitSignIn(person.email, password);
    await this.expectPage('Account');
  }

  /** The alert a refused form shows. */
  formAlert(): Locator {
    return this.page.getByRole('main').getByRole('alert');
  }

  async openAccount(): Promise<void> {
    await this.page.goto('/account');
    await this.expectPage('Account');
  }

  async signOut(): Promise<void> {
    await this.openAccount();
    await this.button('Sign out').click();
    await expect(this.heading('Account')).toBeHidden();
  }
}
