import { expect, type Locator, type Page } from '@playwright/test';

/**
 * The library (accounts.md, H8): its projects, its animations, and their actions — Rename, Move
 * and Duplicate in each row's "Actions for …" menu, Delete as a separate button.
 */
export class LibraryPage {
  readonly animations: Locator;
  readonly projects: Locator;

  constructor(readonly page: Page) {
    this.animations = page.getByRole('region', { name: 'Animations' });
    this.projects = page.getByRole('region', { name: 'Projects' });
  }

  async open(): Promise<void> {
    await this.page.goto('/library');
    await expect(this.page.getByRole('heading', { name: 'Library', level: 1 })).toBeVisible();
  }

  /** An animation of the list, by the link that opens it in the editor. */
  animation(title: string): Locator {
    return this.animations.getByRole('link', { name: title, exact: true });
  }

  /** The animation list shows `titles`, and only them, in whatever order it sorts them. */
  async expectTitles(titles: readonly string[]): Promise<void> {
    const links = this.animations.getByRole('listitem').getByRole('link');
    await expect
      .poll(async () => (await links.allInnerTexts()).map((title) => title.trim()).sort())
      .toEqual([...titles].sort());
  }

  project(name: string): Locator {
    return this.projects.getByRole('link', { name, exact: true });
  }

  async search(query: string): Promise<void> {
    await this.animations
      .getByRole('searchbox', { name: 'Search the animations by title' })
      .fill(query);
    await this.animations.getByRole('button', { name: 'Search', exact: true }).click();
  }

  async rename(title: string, newTitle: string): Promise<void> {
    await this.chooseAction(title, `Rename ${title}`);
    const alert = this.alert('Rename the animation');
    await alert.getByRole('textbox', { name: 'Title' }).fill(newTitle);
    await alert.getByRole('button', { name: 'Rename', exact: true }).click();
    await expect(alert).toBeHidden();
  }

  async duplicate(title: string): Promise<void> {
    await this.chooseAction(title, `Duplicate ${title}`);
  }

  async delete(title: string): Promise<void> {
    await this.animations.getByRole('button', { name: `Delete ${title}`, exact: true }).click();
    const alert = this.alert('Delete this animation?');
    await alert.getByRole('button', { name: 'Delete', exact: true }).click();
    await expect(alert).toBeHidden();
  }

  /** Opens the actions menu of the animation `title`, then chooses its item named `item`. */
  async chooseAction(title: string, item: string): Promise<void> {
    await this.animations
      .getByRole('button', { name: `Actions for ${title}`, exact: true })
      .click();
    await this.animations.getByRole('menuitem', { name: item, exact: true }).click();
  }

  /** An Ionic alert the library opens, by its heading. */
  alert(heading: string): Locator {
    return this.page.locator('ion-alert').filter({ hasText: heading });
  }
}
