import { TestBed } from '@angular/core/testing';
import axe from 'axe-core';
import en from '../../../../../../../i18n/en.json';
import { Shortcuts } from '../../editor/shortcuts';
import type { FakeLibraryStore } from '../testing/fake-library-store';
import {
  configureLibrary,
  SERIOUS_IMPACTS,
  startUnsavedWork,
} from '../testing/library-test-support';
import { SaveButton } from './save-button';
import { SaveFlow } from './save-flow';
import { SavePrompts } from './save-prompts';

/** The catalogue by any key, those of i18n-pending/ included once merged. */
const TEXTS: Readonly<Record<string, string>> = en;

/** Renders the Save button, whose template holds the save dialog. */
async function renderButton(): Promise<{
  store: FakeLibraryStore;
  root: HTMLElement;
  host: HTMLElement;
}> {
  const { store } = await configureLibrary();
  await startUnsavedWork();
  const fixture = TestBed.createComponent(SaveButton);
  await fixture.whenStable();
  // ion-modal moves its presented content to document.body for stacking, once open.
  return { store, root: document.body, host: fixture.nativeElement as HTMLElement };
}

/** Waits for the open dialog to show `selector`, and returns it. */
async function shown<T extends Element>(root: HTMLElement, selector: string): Promise<T> {
  return vi.waitFor(() => {
    const element = root.querySelector<T>(`ion-modal ${selector}`);
    if (!element) throw new Error(`${selector} not shown yet`);
    return element;
  });
}

/** The Save button itself, whichever element draws it. */
function saveButton(host: HTMLElement): HTMLElement {
  const button = host.querySelector<HTMLElement>(':scope > ion-button, :scope > button');
  if (!button) throw new Error('no Save button');
  return button;
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
    const { store, root, host } = await renderButton();
    store.addProject('Sprites');
    saveButton(host).click();
    await shown(root, 'lp-project-picker input[type="radio"]:checked');

    (await shown<HTMLFormElement>(root, 'lp-project-picker form')).requestSubmit();

    await vi.waitFor(() => expect(store.animations).toHaveLength(1));
    await vi.waitFor(() => expect(TestBed.inject(SaveFlow).status()).toBe('saved'));
  });

  it('saves into a new project named in the picker', async () => {
    const { store, root, host } = await renderButton();
    saveButton(host).click();
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

  it('says the usage is unavailable rather than inventing zero', async () => {
    const { root } = await renderButton();

    void TestBed.inject(SavePrompts).showQuota({ usedBytes: 0, limitBytes: null });

    const usage = await shown<HTMLElement>(root, '.usage');
    expect(usage.textContent?.trim()).toBe(TEXTS['library.save.quota.usage_unavailable']);
    expect(usage.textContent).not.toMatch(/\d/);
  });

  it('offers the three ways out of a conflict', async () => {
    const { root } = await renderButton();
    const prompts = TestBed.inject(SavePrompts);

    const answer = prompts.askAboutConflict();
    await shown(root, 'button.lp-button--danger');
    buttonNamed(root, 'Save a copy').click();

    await expect(answer).resolves.toBe('copy');
  });

  it('recommends the copy first, and focuses nothing destructive, on a conflict', async () => {
    const { root } = await renderButton();

    void TestBed.inject(SavePrompts).askAboutConflict();
    await shown(root, 'button.lp-button--danger');

    const buttons = Array.from(root.querySelectorAll<HTMLButtonElement>('ion-modal button'));
    expect(buttons[0]?.textContent?.trim()).toBe('Save a copy');
    expect(buttons[0]?.classList).toContain('lp-button--primary');
    await vi.waitFor(() => expect(document.activeElement).not.toBeNull());
    const focused = document.activeElement;
    expect(focused?.classList.contains('lp-button--danger')).toBe(false);
    expect(root.querySelector('ion-modal button:focus.lp-button--danger')).toBeNull();
  });

  it('writes what reloading and overwriting replace, on their buttons', async () => {
    const { root } = await renderButton();
    void TestBed.inject(SavePrompts).askAboutConflict();
    await shown(root, 'button.lp-button--danger');

    const described = (name: string): string => {
      const id = buttonNamed(root, name).getAttribute('aria-describedby') ?? '';
      return root.querySelector(`[id="${id}"]`)?.textContent?.trim() ?? '';
    };

    expect(described(TEXTS['library.save.conflict.reload'])).toBe(
      TEXTS['library.save.conflict.reload_consequence'],
    );
    expect(described(TEXTS['library.save.conflict.overwrite'])).toBe(
      TEXTS['library.save.conflict.overwrite_consequence'],
    );
  });

  it('answers a visitor who chose to sign in', async () => {
    const { root } = await renderButton();

    const answer = TestBed.inject(SavePrompts).askToSignIn();
    await shown(root, 'button.lp-button--primary');
    buttonNamed(root, 'Sign in').click();

    await expect(answer).resolves.toBe('sign-in');
  });

  describe('has no serious accessibility violation', () => {
    it('asking to sign in', async () => {
      const { root } = await renderButton();
      void TestBed.inject(SavePrompts).askToSignIn();
      await shown(root, 'button.lp-button--primary');

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
      await shown(root, 'button.lp-button--danger');

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
