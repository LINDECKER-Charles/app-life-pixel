import { TestBed } from '@angular/core/testing';
import axe from 'axe-core';
import { LibraryPrompts } from '../actions/library-prompts';
import type { FakeLibraryStore } from '../testing/fake-library-store';
import { configureLibrary, SERIOUS_IMPACTS } from '../testing/library-test-support';
import { LibraryPage } from './library-page';
import { ProjectPage } from './project-page';

interface Prompts {
  askName: ReturnType<typeof vi.fn>;
  pickProject: ReturnType<typeof vi.fn>;
  confirmDestroy: ReturnType<typeof vi.fn>;
}

async function setUp(): Promise<{ store: FakeLibraryStore; prompts: Prompts }> {
  const prompts = { askName: vi.fn(), pickProject: vi.fn(), confirmDestroy: vi.fn() };
  const { store } = await configureLibrary({ provide: LibraryPrompts, useValue: prompts });
  return { store, prompts };
}

async function render<T>(page: new () => T, inputs: Record<string, unknown> = {}) {
  const fixture = TestBed.createComponent(page);
  for (const [name, value] of Object.entries(inputs)) fixture.componentRef.setInput(name, value);
  await fixture.whenStable();
  return fixture.nativeElement as HTMLElement;
}

function texts(root: HTMLElement, selector: string): string[] {
  return Array.from(root.querySelectorAll(selector)).map((node) => node.textContent?.trim() ?? '');
}

function button(root: HTMLElement, label: string): HTMLButtonElement {
  const found = Array.from(root.querySelectorAll('button')).find(
    (candidate) =>
      candidate.getAttribute('aria-label') === label || candidate.textContent?.trim() === label,
  );
  if (!found) throw new Error(`no button "${label}"`);
  return found;
}

function type(root: HTMLElement, selector: string, value: string): void {
  const field = root.querySelector<HTMLInputElement>(selector);
  if (!field) throw new Error(`no field ${selector}`);
  field.value = value;
  field.dispatchEvent(new Event('input'));
}

describe('the library page', () => {
  it('lists the projects with their counts, and every animation opening in the editor', async () => {
    const { store } = await setUp();
    const project = store.addProject('Sprites');
    store.projects[0] = { ...project, animationCount: 2 };
    const walk = store.addAnimation(project.id, 'Walk');
    store.addAnimation(project.id, 'Run');

    const root = await render(LibraryPage);

    await vi.waitFor(() => expect(texts(root, 'lp-project-list .name')).toEqual(['Sprites']));
    expect(texts(root, 'lp-project-list .meta')).toEqual(['2 animations']);
    await vi.waitFor(() => expect(texts(root, 'lp-animation-list .name')).toEqual(['Walk', 'Run']));
    const link = root.querySelector('lp-animation-list a');
    expect(link?.getAttribute('href')).toBe(`/editor/${walk.id}`);
  });

  it('searches the animations by title', async () => {
    const { store } = await setUp();
    const project = store.addProject('Sprites');
    store.addAnimation(project.id, 'Walk');
    store.addAnimation(project.id, 'Run');
    const root = await render(LibraryPage);
    await vi.waitFor(() => expect(texts(root, 'lp-animation-list .name')).toHaveLength(2));

    type(root, '#library-search', 'ru');
    root.querySelector<HTMLFormElement>('form[role="search"]')?.requestSubmit();

    await vi.waitFor(() => expect(texts(root, 'lp-animation-list .name')).toEqual(['Run']));
  });

  it('loads 50 animations at a time, then more on "Load more"', async () => {
    const { store } = await setUp();
    const project = store.addProject('Sprites');
    for (let index = 0; index < 70; index += 1) store.addAnimation(project.id, `A${index}`);
    const root = await render(LibraryPage);
    const items = (): number => root.querySelectorAll('lp-animation-list .item').length;
    await vi.waitFor(() => expect(items()).toBe(50));

    button(root.querySelector('lp-animation-list') as HTMLElement, 'Load more').click();

    await vi.waitFor(() => expect(items()).toBe(70));
    expect(root.querySelector('lp-animation-list .more')).toBeNull();
  });

  it('creates a project, and duplicates it as a copy', async () => {
    const { store } = await setUp();
    const root = await render(LibraryPage);

    type(root, '#library-new-project', 'Heroes');
    root.querySelector<HTMLFormElement>('lp-project-list form')?.requestSubmit();
    await vi.waitFor(() => expect(texts(root, 'lp-project-list .name')).toEqual(['Heroes']));
    button(root, 'Duplicate Heroes').click();

    await vi.waitFor(() =>
      expect(texts(root, 'lp-project-list .name')).toEqual(['Copy of Heroes', 'Heroes']),
    );
    expect(store.projects.map((project) => project.name)).toEqual(['Heroes', 'Copy of Heroes']);
  });

  it('deletes a project only once confirmed', async () => {
    const { store, prompts } = await setUp();
    store.addProject('Sprites');
    const root = await render(LibraryPage);
    await vi.waitFor(() => expect(texts(root, 'lp-project-list .name')).toEqual(['Sprites']));

    prompts.confirmDestroy.mockResolvedValueOnce(false);
    button(root, 'Delete Sprites').click();
    await vi.waitFor(() => expect(prompts.confirmDestroy).toHaveBeenCalledOnce());
    expect(store.projects).toHaveLength(1);

    prompts.confirmDestroy.mockResolvedValueOnce(true);
    button(root, 'Delete Sprites').click();
    await vi.waitFor(() => expect(texts(root, 'lp-project-list .name')).toEqual([]));
    expect(store.projects).toEqual([]);
  });

  it('renames, moves, duplicates and deletes an animation', async () => {
    const { store, prompts } = await setUp();
    const sprites = store.addProject('Sprites');
    const heroes = store.addProject('Heroes');
    store.addAnimation(sprites.id, 'Walk');
    const root = await render(LibraryPage);
    await vi.waitFor(() => expect(texts(root, 'lp-animation-list .name')).toEqual(['Walk']));

    prompts.askName.mockResolvedValueOnce('Stroll');
    button(root, 'Rename Walk').click();
    await vi.waitFor(() => expect(texts(root, 'lp-animation-list .name')).toEqual(['Stroll']));
    prompts.pickProject.mockResolvedValueOnce(heroes.id);
    button(root, 'Move Stroll').click();
    await vi.waitFor(() => expect(store.animations[0]?.projectId).toBe(heroes.id));
    button(root, 'Duplicate Stroll').click();
    await vi.waitFor(() =>
      expect(texts(root, 'lp-animation-list .name')).toEqual(['Copy of Stroll', 'Stroll']),
    );
    prompts.confirmDestroy.mockResolvedValueOnce(true);
    button(root, 'Delete Stroll').click();

    await vi.waitFor(() =>
      expect(texts(root, 'lp-animation-list .name')).toEqual(['Copy of Stroll']),
    );
    expect(store.animations.map((animation) => animation.title)).toEqual(['Copy of Stroll']);
  });

  it('says why an action failed', async () => {
    const { store } = await setUp();
    store.addProject('Sprites');
    const root = await render(LibraryPage);
    await vi.waitFor(() => expect(texts(root, 'lp-project-list .name')).toEqual(['Sprites']));

    store.failNext('service.unavailable');
    button(root, 'Duplicate Sprites').click();

    await vi.waitFor(() =>
      expect(root.querySelector('lp-project-list [role="alert"]')?.textContent).toContain(
        'unavailable',
      ),
    );
  });

  it('has no serious accessibility violation', async () => {
    const { store } = await setUp();
    const project = store.addProject('Sprites');
    for (let index = 0; index < 60; index += 1) store.addAnimation(project.id, `A${index}`);
    const root = await render(LibraryPage);
    await vi.waitFor(() => expect(root.querySelector('lp-animation-list .more')).not.toBeNull());

    const { violations } = await axe.run(root);

    expect(
      violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? '')),
    ).toEqual([]);
  });
});

describe('the project page', () => {
  it('names the project and lists its animations only', async () => {
    const { store } = await setUp();
    const sprites = store.addProject('Sprites');
    const heroes = store.addProject('Heroes');
    store.addAnimation(sprites.id, 'Walk');
    store.addAnimation(heroes.id, 'Jump');

    const root = await render(ProjectPage, { projectId: heroes.id });

    await vi.waitFor(() => expect(root.querySelector('h1')?.textContent?.trim()).toBe('Heroes'));
    await vi.waitFor(() => expect(texts(root, 'lp-animation-list .name')).toEqual(['Jump']));
    expect(root.querySelector('form[role="search"]')).toBeNull();
  });

  it('says so when the project does not exist', async () => {
    await setUp();

    const root = await render(ProjectPage, { projectId: 'gone' });

    await vi.waitFor(() =>
      expect(root.querySelector('[role="alert"]')?.textContent).toContain('does not exist'),
    );
  });

  it('lets a moved animation leave the project', async () => {
    const { store, prompts } = await setUp();
    const sprites = store.addProject('Sprites');
    const heroes = store.addProject('Heroes');
    store.addAnimation(sprites.id, 'Walk');
    const root = await render(ProjectPage, { projectId: sprites.id });
    await vi.waitFor(() => expect(texts(root, 'lp-animation-list .name')).toEqual(['Walk']));

    prompts.pickProject.mockResolvedValueOnce(heroes.id);
    button(root, 'Move Walk').click();

    await vi.waitFor(() => expect(texts(root, 'lp-animation-list .name')).toEqual([]));
  });

  it('has no serious accessibility violation', async () => {
    const { store } = await setUp();
    const project = store.addProject('Sprites');
    store.addAnimation(project.id, 'Walk');
    const root = await render(ProjectPage, { projectId: project.id });
    await vi.waitFor(() => expect(texts(root, 'lp-animation-list .name')).toEqual(['Walk']));

    const { violations } = await axe.run(root);

    expect(
      violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? '')),
    ).toEqual([]);
  });
});
