import { TestBed } from '@angular/core/testing';
import { Router } from '@angular/router';
import { EngineStore } from '../../engine/engine-store';
import { MockEditorEngine } from '../../engine/testing/mock-editor-engine';
import { CurrentAnimation } from '../current-animation';
import { AnimationLoader } from '../open/animation-loader';
import { SaveFlow } from '../save/save-flow';
import { SavePrompts } from '../save/save-prompts';
import type { FakeLibraryStore } from '../testing/fake-library-store';
import { configureLibrary } from '../testing/library-test-support';
import { LibraryChangedPrompt, type LibraryChangedChoice } from './library-changed-prompt';
import { LibraryChanges } from './library-changes';
import { OpenAnimationRefresh } from './open-animation-refresh';

/** The bytes of a document titled `title`, from an engine of its own. */
async function documentTitled(title: string): Promise<Uint8Array> {
  const engine = new MockEditorEngine();
  await engine.create({ title, width: 8, height: 8, layerName: 'Base' });
  return engine.serialize();
}

describe('OpenAnimationRefresh', () => {
  let ask: ReturnType<typeof vi.fn<() => Promise<LibraryChangedChoice>>>;

  /** The animation "Saved", open in the editor at version 1, with or without unsaved work. */
  async function openSaved({ unsavedWork }: { unsavedWork: boolean }) {
    ask = vi.fn<() => Promise<LibraryChangedChoice>>();
    const bed = await configureLibrary({ provide: LibraryChangedPrompt, useValue: { ask } });
    const project = bed.store.addProject('Sprites');
    const saved = bed.store.addAnimation(project.id, 'Saved', await documentTitled('Saved'));
    await TestBed.inject(AnimationLoader).load(saved.id);
    const engine = TestBed.inject(EngineStore);
    if (unsavedWork) await engine.apply({ kind: 'setTitle', title: 'Mine' });
    const change = { projectIds: [project.id], animationIds: [saved.id] };
    return { ...bed, saved, engine, change };
  }

  /** An agent's write: the document on disk moves to version 2. */
  async function changeOnDisk(store: FakeLibraryStore, id: string): Promise<void> {
    await store.saveDocument(id, await documentTitled('By agent'), 1);
  }

  it('reloads the open animation when it holds no unsaved work', async () => {
    const { store, saved, engine, change } = await openSaved({ unsavedWork: false });
    await changeOnDisk(store, saved.id);
    TestBed.inject(OpenAnimationRefresh).follow();
    TestBed.inject(LibraryChanges).announce(change);

    await vi.waitFor(() => expect(engine.document()?.title).toBe('By agent'));
    expect(TestBed.inject(CurrentAnimation).state()).toMatchObject({ id: saved.id, version: 2 });
    expect(ask).not.toHaveBeenCalled();
  });

  it('asks before reloading over unsaved work, and reloads when told', async () => {
    const { store, saved, engine, change } = await openSaved({ unsavedWork: true });
    await changeOnDisk(store, saved.id);
    ask.mockResolvedValue('reload');
    await TestBed.inject(OpenAnimationRefresh).onChange(change);

    expect(ask).toHaveBeenCalledOnce();
    expect(engine.document()?.title).toBe('By agent');
    expect(engine.hasUnsavedWork()).toBe(false);
  });

  it('keeps my work, and the next save meets the conflict dialog', async () => {
    const { store, saved, engine, change } = await openSaved({ unsavedWork: true });
    await changeOnDisk(store, saved.id);
    ask.mockResolvedValue('keep');
    await TestBed.inject(OpenAnimationRefresh).onChange(change);

    expect(engine.document()?.title).toBe('Mine');
    const prompts = TestBed.inject(SavePrompts);
    const saving = TestBed.inject(SaveFlow).save();
    await vi.waitFor(() => expect(prompts.current()?.kind).toBe('conflict'));
    prompts.dismiss();
    await saving;
  });

  it('saves my work as a copy, which the editor then follows', async () => {
    const { store, saved, engine, change } = await openSaved({ unsavedWork: true });
    await changeOnDisk(store, saved.id);
    ask.mockResolvedValue('copy');
    await TestBed.inject(OpenAnimationRefresh).onChange(change);

    const copy = store.animations.find(({ id }) => id !== saved.id);
    expect(copy?.title).toMatch(/^Copy of /);
    expect(TestBed.inject(CurrentAnimation).id()).toBe(copy?.id);
    expect(engine.hasUnsavedWork()).toBe(false);
    expect(TestBed.inject(Router).url).toBe(`/editor/${copy?.id}`);
  });

  it('ignores the app’s own saves and changes to other animations', async () => {
    const { store, saved, engine, change } = await openSaved({ unsavedWork: true });
    const refresh = TestBed.inject(OpenAnimationRefresh);
    await refresh.onChange(change);
    await changeOnDisk(store, saved.id);
    await refresh.onChange({ projectIds: [saved.projectId], animationIds: ['another'] });

    expect(ask).not.toHaveBeenCalled();
    expect(engine.document()?.title).toBe('Mine');
  });
});
