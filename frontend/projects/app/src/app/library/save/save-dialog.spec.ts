import { TestBed } from '@angular/core/testing';
import axe from 'axe-core';
import { Shortcuts } from '../../editor/shortcuts';
import type { FakeLibraryStore } from '../testing/fake-library-store';
import {
  configureLibrary,
  SERIOUS_IMPACTS,
  startUnsavedWork,
} from '../testing/library-test-support';
import { SaveButton } from './save-button';
import { SavePrompts } from './save-prompts';

/** Renders the Save button, whose template holds the save dialog. */
async function renderButton(): Promise<{ store: FakeLibraryStore; root: HTMLElement }> {
  const { store } = await configureLibrary();
  await startUnsavedWork();
  const fixture = TestBed.createComponent(SaveButton);
  await fixture.whenStable();
  // ion-modal moves its presented content to document.body for stacking, once open.
  return { store, root: document.body };
}

/** Waits for the open dialog to show `selector`, and returns it. */
async function shown<T extends Element>(root: HTMLElement, selector: string): Promise<T> {
  return vi.waitFor(() => {
    const element = root.querySelector<T>(`ion-modal ${selector}`);
    if (!element) throw new Error(`${selector} not shown yet`);
    return element;
  });
}

function buttonNamed(root: HTMLElement, text: string): HTMLButtonElement {
  const button = Array.from(root.querySelectorAll<HTMLButtonElement>('ion-modal button')).find(
    (candidate) => candidate.textContent?.trim() === text,
  );
  if (!button) throw new Error(`no button "${text}"`);
  return button;
}

async function seriousViolations(root: HTMLElement): Promise<axe.Result[]> {
  const { violations } = await axe.run(root);
  return violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? ''));
}

describe('the save button and its dialog', () => {
  afterEach(() => document.body.replaceChildren());

  it('saves on Ctrl/⌘ S, asking for a project the first time', async () => {
    const { store } = await renderButton();
    const project = store.addProject('Sprites');
    const event = new KeyboardEvent('keydown', { key: 's', metaKey: true, cancelable: true });

    TestBed.inject(Shortcuts).handle(event);

    expect(event.defaultPrevented).toBe(true);
    const option = await shown<HTMLInputElement>(document.body, `input[value="${project.id}"]`);
    await vi.waitFor(() => expect(option.checked).toBe(true));
  });

  it('saves into the project picked', async () => {
    const { store, root } = await renderButton();
    store.addProject('Sprites');
    root.querySelector<HTMLElement>('ion-button')?.click();
    await shown(root, 'lp-project-picker input[type="radio"]:checked');

    (await shown<HTMLFormElement>(root, 'lp-project-picker form')).requestSubmit();

    await vi.waitFor(() => expect(store.animations).toHaveLength(1));
    await vi.waitFor(() => expect(root.textContent).toContain('Saved'));
  });

  it('saves into a new project named in the picker', async () => {
    const { store, root } = await renderButton();
    root.querySelector<HTMLElement>('ion-button')?.click();
    const name = await shown<HTMLInputElement>(root, 'lp-project-picker input[name="name"]');

    name.value = 'Heroes';
    name.dispatchEvent(new Event('input'));
    await vi.waitFor(() => expect(buttonNamed(root, 'Save here').disabled).toBe(false));
    buttonNamed(root, 'Save here').click();

    await vi.waitFor(() =>
      expect(store.projects.map((project) => project.name)).toEqual(['Heroes']),
    );
    expect(store.animations).toHaveLength(1);
  });

  it('shows the usage and the limit, with a link to the library', async () => {
    const { root } = await renderButton();

    void TestBed.inject(SavePrompts).showQuota({ usedBytes: 990, limitBytes: 1000 });

    const link = await shown<HTMLAnchorElement>(root, 'a[href="/library"]');
    expect(link.textContent?.trim()).toBe('Open the library');
    expect(root.querySelector('ion-modal')?.textContent).toContain('990 of 1,000 bytes');
  });

  it('offers the three ways out of a conflict', async () => {
    const { root } = await renderButton();
    const prompts = TestBed.inject(SavePrompts);

    const answer = prompts.askAboutConflict();
    await shown(root, 'button.danger');
    buttonNamed(root, 'Save a copy').click();

    await expect(answer).resolves.toBe('copy');
  });

  it('answers a visitor who chose to sign in', async () => {
    const { root } = await renderButton();

    const answer = TestBed.inject(SavePrompts).askToSignIn();
    await shown(root, 'button.primary');
    buttonNamed(root, 'Sign in').click();

    await expect(answer).resolves.toBe('sign-in');
  });

  describe('has no serious accessibility violation', () => {
    it('asking to sign in', async () => {
      const { root } = await renderButton();
      void TestBed.inject(SavePrompts).askToSignIn();
      await shown(root, 'button.primary');

      expect(await seriousViolations(root)).toEqual([]);
    });

    it('asking for a project', async () => {
      const { store, root } = await renderButton();
      store.addProject('Sprites');
      void TestBed.inject(SavePrompts).askForProject();
      await shown(root, 'lp-project-picker input[type="radio"]:checked');

      expect(await seriousViolations(root)).toEqual([]);
    });

    it('on a conflict', async () => {
      const { root } = await renderButton();
      void TestBed.inject(SavePrompts).askAboutConflict();
      await shown(root, 'button.danger');

      expect(await seriousViolations(root)).toEqual([]);
    });

    it('with the quota reached', async () => {
      const { root } = await renderButton();
      void TestBed.inject(SavePrompts).showQuota({ usedBytes: 990, limitBytes: 1000 });
      await shown(root, 'a[href="/library"]');

      expect(await seriousViolations(root)).toEqual([]);
    });

    it('on a failure', async () => {
      const { root } = await renderButton();
      void TestBed.inject(SavePrompts).showFailure({ code: 'service.unavailable', params: {} });
      await shown(root, '[role="alert"]');

      expect(await seriousViolations(root)).toEqual([]);
    });
  });
});
