import { TestBed } from '@angular/core/testing';
import { Router } from '@angular/router';
import { EDITOR_ENGINE } from '../../engine/editor-engine';
import { EngineStore } from '../../engine/engine-store';
import { MockEditorEngine } from '../../engine/testing/mock-editor-engine';
import { CurrentAnimation } from '../current-animation';
import type { FakeLibraryStore } from '../testing/fake-library-store';
import { configureLibrary, startUnsavedWork } from '../testing/library-test-support';
import { SaveFlow } from './save-flow';
import { SavePrompts, type SavePrompt } from './save-prompts';

/** The bytes of a document titled `title`, from an engine of its own. */
async function documentTitled(title: string): Promise<Uint8Array> {
  const engine = new MockEditorEngine();
  await engine.create({ title, width: 8, height: 8, layerName: 'Base' });
  return engine.serialize();
}

/** Waits for the save dialog to ask `kind`, and returns the question. */
async function question(kind: SavePrompt['kind']): Promise<SavePrompt> {
  const prompts = TestBed.inject(SavePrompts);
  return vi.waitFor(() => {
    const prompt = prompts.current();
    if (prompt?.kind !== kind) throw new Error(`no ${kind} question yet`);
    return prompt;
  });
}

/** Saves the work once over `store`'s project, as a signed-in person's first save does. */
async function firstSave(store: FakeLibraryStore): Promise<string> {
  const project = store.addProject('Sprites');
  const saving = TestBed.inject(SaveFlow).save();
  await question('project');
  TestBed.inject(SavePrompts).answerProject({ projectId: project.id });
  await saving;
  const id = TestBed.inject(CurrentAnimation).id();
  if (id === null) throw new Error('the first save saved nothing');
  return id;
}

describe('SaveFlow', () => {
  it('asks a visitor to sign in, then saves once back in the editor with the work intact', async () => {
    const { store, signedIn } = await configureLibrary();
    signedIn.set(false);
    const engine = await startUnsavedWork('Walk cycle');
    const router = TestBed.inject(Router);
    await router.navigateByUrl('/editor');
    store.addProject('Sprites');

    const saving = TestBed.inject(SaveFlow).save();
    await question('sign-in');
    TestBed.inject(SavePrompts).answerSignIn('sign-in');
    await saving;

    expect(router.url).toBe('/sign-in?returnUrl=%2Feditor');
    signedIn.set(true);
    await router.navigateByUrl('/editor');
    await question('project');
    expect(engine.document()?.title).toBe('Walk cycle');
    expect(engine.hasUnsavedWork()).toBe(true);
  });

  it('sends a visitor who chose to sign up to the sign-up page', async () => {
    const { signedIn } = await configureLibrary();
    signedIn.set(false);
    await startUnsavedWork();
    const router = TestBed.inject(Router);
    await router.navigateByUrl('/editor');

    const saving = TestBed.inject(SaveFlow).save();
    await question('sign-in');
    TestBed.inject(SavePrompts).answerSignIn('sign-up');
    await saving;

    expect(router.url).toBe('/sign-up?returnUrl=%2Feditor');
  });

  it('saves unsaved work in the project picked, then follows the saved animation', async () => {
    const { store } = await configureLibrary();
    const engine = await startUnsavedWork();

    const id = await firstSave(store);

    expect(store.animations.map((animation) => animation.id)).toEqual([id]);
    expect(engine.hasUnsavedWork()).toBe(false);
    expect(TestBed.inject(Router).url).toBe(`/editor/${id}`);
    expect(TestBed.inject(SaveFlow).status()).toBe('saved');
  });

  it('creates the project a first save names', async () => {
    const { store } = await configureLibrary();
    await startUnsavedWork();

    const saving = TestBed.inject(SaveFlow).save();
    await question('project');
    TestBed.inject(SavePrompts).answerProject({ newProjectName: 'Heroes' });
    await saving;

    expect(store.projects.map((project) => project.name)).toEqual(['Heroes']);
    expect(store.animations[0]?.projectId).toBe(store.projects[0]?.id);
  });

  it('keeps the work unsaved when the first save is cancelled', async () => {
    const { store } = await configureLibrary();
    const engine = await startUnsavedWork();

    const saving = TestBed.inject(SaveFlow).save();
    await question('project');
    TestBed.inject(SavePrompts).dismiss();
    await saving;

    expect(store.animations).toEqual([]);
    expect(engine.hasUnsavedWork()).toBe(true);
  });

  it('saves a saved animation under its version', async () => {
    const { store } = await configureLibrary();
    const engine = await startUnsavedWork();
    const id = await firstSave(store);
    await engine.apply({ kind: 'setTitle', title: 'Run cycle' });

    await TestBed.inject(SaveFlow).save();

    expect(store.animations[0]?.version).toBe(2);
    expect(TestBed.inject(CurrentAnimation).state()).toMatchObject({ id, version: 2 });
    expect(engine.hasUnsavedWork()).toBe(false);
  });

  describe('when the animation changed elsewhere', () => {
    async function conflict(): Promise<{
      store: FakeLibraryStore;
      engine: EngineStore;
      id: string;
    }> {
      const { store } = await configureLibrary();
      const engine = await startUnsavedWork('Mine');
      const id = await firstSave(store);
      await store.saveDocument(id, await documentTitled('Theirs'), 1);
      await engine.apply({ kind: 'setTitle', title: 'Mine again' });
      return { store, engine, id };
    }

    async function choose(choice: 'reload' | 'overwrite' | 'copy'): Promise<void> {
      const saving = TestBed.inject(SaveFlow).save();
      await question('conflict');
      TestBed.inject(SavePrompts).answerConflict(choice);
      await saving;
    }

    it('reloads the saved version', async () => {
      const { engine, id } = await conflict();

      await choose('reload');

      expect(engine.document()?.title).toBe('Theirs');
      expect(engine.hasUnsavedWork()).toBe(false);
      expect(TestBed.inject(CurrentAnimation).state()).toMatchObject({ id, version: 2 });
    });

    it('overwrites it, after reading its current version', async () => {
      const { store, engine, id } = await conflict();

      await choose('overwrite');

      expect(store.animations[0]?.version).toBe(3);
      expect(store.documents.get(id)).toEqual(await TestBed.inject(EDITOR_ENGINE).serialize());
      expect(engine.hasUnsavedWork()).toBe(false);
    });

    it('saves the work as a copy, which the editor then follows', async () => {
      const { store, engine, id } = await conflict();

      await choose('copy');

      const copy = store.animations[1];
      expect(copy?.title).toBe('Copy of Saved');
      expect(store.animations[0]?.version).toBe(2);
      expect(TestBed.inject(CurrentAnimation).id()).toBe(copy?.id);
      expect(TestBed.inject(CurrentAnimation).id()).not.toBe(id);
      expect(engine.document()?.title).toBe('Copy of Saved');
      expect(engine.hasUnsavedWork()).toBe(false);
    });
  });

  it('shows the usage and the limit when the quota is reached, keeping the work', async () => {
    const { store } = await configureLibrary();
    const engine = await startUnsavedWork();
    const id = await firstSave(store);
    await engine.apply({ kind: 'setTitle', title: 'Longer' });
    store.failNext('quota.storage_exceeded', { used: 990, limit: 1000, requested: 20 });

    const saving = TestBed.inject(SaveFlow).save();
    const prompt = await question('quota');
    expect(prompt).toMatchObject({ usage: { usedBytes: 990, limitBytes: 1000 } });
    TestBed.inject(SavePrompts).dismiss();
    await saving;

    expect(engine.hasUnsavedWork()).toBe(true);
    expect(TestBed.inject(CurrentAnimation).state()).toMatchObject({ id, version: 1 });
  });

  it('says why a save failed, keeping the work', async () => {
    const { store } = await configureLibrary();
    const engine = await startUnsavedWork();
    await firstSave(store);
    await engine.apply({ kind: 'setTitle', title: 'Longer' });
    store.failNext('service.unavailable');

    const saving = TestBed.inject(SaveFlow).save();
    const prompt = await question('failure');
    expect(prompt).toMatchObject({ failure: { code: 'service.unavailable' } });
    TestBed.inject(SavePrompts).dismiss();
    await saving;

    expect(engine.hasUnsavedWork()).toBe(true);
    expect(TestBed.inject(SaveFlow).status()).toBe('idle');
  });
});
