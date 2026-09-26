import { expect, type Locator } from '@playwright/test';
import { COLOR, type Pixel } from '../../editor/drawing';
import { EditorPage } from '../../editor/editor-page';
import { expectSettled } from './outlet';

/** The size of the journeys' animations: at the default zoom of 8 it fits any viewport. */
const SIDE = 16;
/** A saved animation's route: `/editor/<its id>`. */
const SAVED_ROUTE = /\/editor\/[0-9a-f-]{36}$/;

/** The strokes drawn in this worker, by every page: two pages may draw on one document. */
let strokes = 0;

/** A short stroke on a row of its own, so that each drawing changes the document. */
function nextStroke(): Pixel[] {
  const row = (strokes++ * 2 + 1) % SIDE;
  return [0, 1, 2, 3].map((x) => ({ x: x + 2, y: row }));
}

/**
 * U6's editor page (e2e/editor/) with what the hosted app adds to it (accounts.md, H8): saving,
 * and the dialogs saving asks through — sign in, the project, a conflict, the quota.
 */
export class HostedEditor extends EditorPage {
  /** The dialog saving shows, by its title. */
  saveDialog(title: string): Locator {
    return this.page.getByRole('dialog', { name: title, exact: true });
  }

  /** Opens `/editor` and creates an animation titled `title` in the new-animation dialog. */
  async createAnimation(title: string): Promise<void> {
    await this.open();
    await this.fillNewAnimation(title);
  }

  /** Starts a new animation from the editor's "New" button. */
  async newAnimation(title: string): Promise<void> {
    await this.button('New').click();
    await expect(this.newDialog).toBeVisible();
    await this.fillNewAnimation(title);
  }

  private async fillNewAnimation(title: string): Promise<void> {
    await this.textbox('Title').fill(title);
    await this.spinbutton('Width').fill(String(SIDE));
    await this.spinbutton('Height').fill(String(SIDE));
    await this.button('Create').click();
    await this.expectReady();
    await this.settled();
  }

  /** Waits for the page transition: a new route opens a second editor over the first. */
  async settled(): Promise<void> {
    await expectSettled(this.page);
  }

  /** Draws a black pencil stroke on the current frame. */
  async draw(): Promise<void> {
    await this.tool('Pencil').click();
    await this.swatch(COLOR.black.index).click();
    await this.drag(nextStroke());
    await expect(this.button('Undo')).toBeEnabled();
  }

  /** Clicks Save. */
  async save(): Promise<void> {
    await this.button('Save').click();
  }

  /** In the project dialog of a first save: a new project named `name`. */
  async saveInNewProject(name: string): Promise<void> {
    await expect(this.saveDialog('Save the animation')).toBeVisible();
    await this.radio('A new project').check();
    await this.page.getByRole('textbox', { name: 'Project name' }).fill(name);
    await this.button('Save here').click();
  }

  /** In the project dialog of a first save: the existing project `name`. */
  async saveInProject(name: string): Promise<void> {
    await expect(this.saveDialog('Save the animation')).toBeVisible();
    await this.radio(name).check();
    await this.button('Save here').click();
  }

  /** The work is saved: the status says so, and the route names the animation. */
  async expectSaved(): Promise<void> {
    await expect(this.page.getByRole('status').filter({ hasText: /^\s*Saved\s*$/ })).toBeVisible();
    await expect(this.page).toHaveURL(SAVED_ROUTE);
    await this.settled();
  }

  /** The id of the saved animation the editor holds. */
  savedId(): string {
    const id = new URL(this.page.url()).pathname.split('/').pop();
    if (!id || !SAVED_ROUTE.test(this.page.url())) throw new Error('No saved animation is open.');
    return id;
  }

  /** The animation's title, as the header's field shows it. */
  titleField(): Locator {
    return this.textbox('Animation title');
  }
}
