import { TestBed } from '@angular/core/testing';
import axe from 'axe-core';
import { configureLibrary, SERIOUS_IMPACTS } from '../../library/testing/library-test-support';
import { NewAnimationFlow } from '../new-animation/new-animation-flow';
import { EditorWelcome } from './editor-welcome';

/** Renders the welcome for a visitor, or for a signed-in person. */
async function render(signedIn: boolean): Promise<HTMLElement> {
  const library = await configureLibrary();
  library.signedIn.set(signedIn);
  const fixture = TestBed.createComponent(EditorWelcome);
  await fixture.whenStable();
  return fixture.nativeElement as HTMLElement;
}

describe('the editor welcome', () => {
  afterEach(() => document.body.replaceChildren());

  it('says what can be made, with Pip beside it as decoration', async () => {
    const root = await render(false);

    expect(root.querySelector('h2')?.textContent?.trim()).toBe(
      'A little world, one pixel at a time.',
    );
    expect(root.textContent).toContain('Draw your first frame, bring it to life, then export it.');
    expect(root.querySelector('img')?.getAttribute('alt')).toBe('');
  });

  it('opens the new-animation dialog from "Create animation"', async () => {
    const root = await render(false);
    const create = root.querySelector<HTMLButtonElement>('button.lp-button--primary');
    expect(create?.textContent?.trim()).toBe('Create animation');

    create?.click();

    await vi.waitFor(() => expect(TestBed.inject(NewAnimationFlow).isOpen()).toBe(true));
  });

  it('tells a visitor that no account is needed, and that unsaved work does not last', async () => {
    const root = await render(false);

    expect(root.textContent).toContain('You can draw and export without an account.');
    expect(root.textContent).toContain('Unsaved work is lost when this page closes or reloads.');
  });

  it('spares a signed-in person the visitor notice', async () => {
    const root = await render(true);

    expect(root.textContent).not.toContain('without an account');
  });

  it('has no serious accessibility violation', async () => {
    const root = await render(false);

    const { violations } = await axe.run(root);
    expect(violations.filter((v) => SERIOUS_IMPACTS.includes(v.impact ?? ''))).toEqual([]);
  });
});
