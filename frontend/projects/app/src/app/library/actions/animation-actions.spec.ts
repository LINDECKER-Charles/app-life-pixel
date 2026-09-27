import { TestBed } from '@angular/core/testing';
import { CurrentAnimation } from '../current-animation';
import { PagedList } from '../lists/paged-list';
import type { AnimationSummary } from '../library-types';
import { configureLibrary, startUnsavedWork } from '../testing/library-test-support';
import { AnimationActions } from './animation-actions';
import { LibraryPrompts } from './library-prompts';

describe('AnimationActions on the editor’s animation', () => {
  async function setUp() {
    const prompts = { askName: vi.fn(), pickProject: vi.fn(), confirmDestroy: vi.fn() };
    const { store } = await configureLibrary({ provide: LibraryPrompts, useValue: prompts });
    const project = store.addProject('Sprites');
    const saved = store.addAnimation(project.id, 'Walk');
    const engine = await startUnsavedWork('Walk');
    await engine.markSaved();
    TestBed.inject(CurrentAnimation).setSaved(saved);
    const list = new PagedList<AnimationSummary>((page) => store.listAnimations({}, page));
    await list.reload();
    return { store, prompts, engine, saved, list };
  }

  it('follows a rename: the new version and title, the work still saved', async () => {
    const { prompts, engine, saved, list } = await setUp();
    prompts.askName.mockResolvedValue('Stroll');

    await TestBed.inject(AnimationActions).rename(saved, list);

    expect(TestBed.inject(CurrentAnimation).state()).toMatchObject({ id: saved.id, version: 2 });
    expect(engine.document()?.title).toBe('Stroll');
    expect(engine.hasUnsavedWork()).toBe(false);
  });

  it('keeps the work as unsaved once its animation is deleted', async () => {
    const { prompts, engine, saved, list } = await setUp();
    prompts.confirmDestroy.mockResolvedValue(true);

    await TestBed.inject(AnimationActions).delete(saved, list);

    expect(TestBed.inject(CurrentAnimation).state()).toEqual({ kind: 'unsaved' });
    expect(engine.document()?.title).toBe('Walk');
  });

  it('follows a move to another project', async () => {
    const { store, prompts, saved, list } = await setUp();
    const heroes = store.addProject('Heroes');
    prompts.pickProject.mockResolvedValue(heroes.id);

    await TestBed.inject(AnimationActions).move(saved, list);

    expect(TestBed.inject(CurrentAnimation).state()).toMatchObject({ projectId: heroes.id });
  });
});
