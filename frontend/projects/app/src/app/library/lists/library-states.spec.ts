import { TestBed } from '@angular/core/testing';
import { Router } from '@angular/router';
import en from '../../../../../../../i18n/en.json';
import { NewAnimationFlow } from '../../editor/new-animation/new-animation-flow';
import { LibraryPrompts } from '../actions/library-prompts';
import { LibraryPage } from '../pages/library-page';
import { ProjectPage } from '../pages/project-page';
import type { FakeLibraryStore } from '../testing/fake-library-store';
import { configureLibrary } from '../testing/library-test-support';

// Texts of i18n-pending/l8a, merged into the catalogues at integration.
const CLEAR = 'Clear the search';

async function setUp(): Promise<{ store: FakeLibraryStore; start: ReturnType<typeof vi.fn> }> {
  const start = vi.fn().mockResolvedValue(undefined);
  const prompts = { askName: vi.fn(), pickProject: vi.fn(), confirmDestroy: vi.fn() };
  const { store } = await configureLibrary(
    { provide: LibraryPrompts, useValue: prompts },
    { provide: NewAnimationFlow, useValue: { start } },
  );
  return { store, start };
}

async function render<T>(page: new () => T, inputs: Record<string, unknown> = {}) {
  const fixture = TestBed.createComponent(page);
  for (const [name, value] of Object.entries(inputs)) fixture.componentRef.setInput(name, value);
  await fixture.whenStable();
  return fixture.nativeElement as HTMLElement;
}

function animations(root: HTMLElement): HTMLElement {
  return root.querySelector('lp-animation-list') as HTMLElement;
}

function titles(root: HTMLElement): string[] {
  return Array.from(root.querySelectorAll('lp-animation-list .name')).map(
    (link) => link.textContent?.trim() ?? '',
  );
}

function buttonNamed(root: HTMLElement, label: string): HTMLButtonElement | undefined {
  return Array.from(root.querySelectorAll('button')).find(
    (candidate) => candidate.textContent?.trim() === label,
  );
}

async function searchFor(root: HTMLElement, query: string): Promise<void> {
  const field = root.querySelector<HTMLInputElement>('#library-search');
  if (!field) throw new Error('no search field');
  field.value = query;
  field.dispatchEvent(new Event('input'));
  root.querySelector<HTMLFormElement>('form[role="search"]')?.requestSubmit();
}

describe('the library states', () => {
  it('shows an empty library with Pip, apart from a search or a failure', async () => {
    await setUp();

    const root = await render(LibraryPage);

    const list = animations(root);
    await vi.waitFor(() =>
      expect(list.querySelector('lp-empty-state h3')?.textContent).toContain(
        'No saved animation yet',
      ),
    );
    expect(list.querySelector('lp-empty-state img')?.getAttribute('alt')).toBe('');
    expect(list.querySelector('[role="alert"]')).toBeNull();
    expect(buttonNamed(list, CLEAR)).toBeUndefined();
  });

  it('says a search found nothing, without Pip, and clears it back to every animation', async () => {
    const { store } = await setUp();
    const project = store.addProject('Sprites');
    store.addAnimation(project.id, 'Walk');
    const root = await render(LibraryPage);
    await vi.waitFor(() => expect(titles(root)).toEqual(['Walk']));

    await searchFor(root, 'jump');

    const list = animations(root);
    await vi.waitFor(() =>
      expect(list.querySelector('.no-results [role="status"]')?.textContent).toContain(
        'No animation matches “jump”.',
      ),
    );
    expect(list.querySelector('lp-empty-state')).toBeNull();
    expect(list.querySelector('img')).toBeNull();

    buttonNamed(list, CLEAR)?.click();

    await vi.waitFor(() => expect(titles(root)).toEqual(['Walk']));
    expect(list.querySelector<HTMLInputElement>('#library-search')?.value).toBe('');
    expect(document.activeElement?.id).toBe('library-search');
  });

  it('shows a failure to read the library as an error with Retry, never as empty', async () => {
    const { store } = await setUp();
    const project = store.addProject('Sprites');
    store.addAnimation(project.id, 'Walk');
    // The project list reads first, then the animation list.
    store.failNext('service.unavailable');
    store.failNext('service.unavailable');

    const root = await render(LibraryPage);

    const list = animations(root);
    await vi.waitFor(() =>
      expect(list.querySelector('[role="alert"]')?.textContent).toContain('unavailable'),
    );
    expect(list.querySelector('lp-empty-state')).toBeNull();
    expect(list.querySelector('.no-results')).toBeNull();

    buttonNamed(list, en['common.retry'])?.click();

    await vi.waitFor(() => expect(titles(root)).toEqual(['Walk']));
    expect(list.querySelector('[role="alert"]')).toBeNull();
  });

  it('says it is reading before the first page arrives', async () => {
    const { store } = await setUp();
    vi.spyOn(store, 'listAnimations').mockReturnValue(new Promise(() => undefined));

    const root = await render(LibraryPage);

    expect(animations(root).querySelector('[role="status"]')?.textContent).toContain(
      en['common.loading'],
    );
    expect(animations(root).querySelector('lp-empty-state')).toBeNull();
  });

  it('offers to create an animation in an empty project', async () => {
    const { store, start } = await setUp();
    const project = store.addProject('Sprites');
    const root = await render(ProjectPage, { projectId: project.id });

    const create = await vi.waitFor(() => {
      const found = buttonNamed(animations(root), 'Create an animation');
      if (!found) throw new Error('no create button');
      return found;
    });
    expect(animations(root).querySelector('lp-empty-state h2')?.textContent).toContain(
      'No animation in this project yet',
    );
    create.click();

    await vi.waitFor(() => expect(start).toHaveBeenCalledOnce());
    expect(TestBed.inject(Router).url).toBe('/editor');
  });

  it('puts the actions of an animation in its menu, and Delete apart', async () => {
    const { store } = await setUp();
    const project = store.addProject('Sprites');
    store.addAnimation(project.id, 'Walk');
    const root = await render(LibraryPage);
    await vi.waitFor(() => expect(titles(root)).toEqual(['Walk']));

    const row = animations(root).querySelector('.item') as HTMLElement;
    const menu = row.querySelector('[role="menu"]');
    const items = Array.from(menu?.querySelectorAll('[role="menuitem"]') ?? []);
    expect(menu?.getAttribute('aria-label')).toBe('Actions for Walk');
    expect(items.map((item) => item.getAttribute('aria-label'))).toEqual([
      'Rename Walk',
      'Move Walk',
      'Duplicate Walk',
    ]);
    const remove = row.querySelector('button[aria-label="Delete Walk"]');
    expect(remove).not.toBeNull();
    expect(menu?.contains(remove ?? null)).toBe(false);
  });
});
