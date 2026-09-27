import { TestBed } from '@angular/core/testing';
import { EngineStore } from '../../../engine/engine-store';
import type { FakeLibraryStore } from '../../testing/fake-library-store';
import { configureLibrary, startUnsavedWork } from '../../testing/library-test-support';
import { SaveFlow } from '../save-flow';
import { SavePrompts, type SavePrompt } from '../save-prompts';
import { SaveStatePill } from './save-state-pill';

/** Renders the pill, and returns a reader of the text its live region says. */
async function renderPill(): Promise<() => string> {
  const fixture = TestBed.createComponent(SaveStatePill);
  await fixture.whenStable();
  const status = (fixture.nativeElement as HTMLElement).querySelector('[role="status"]');
  return () => {
    fixture.detectChanges();
    return status?.textContent?.trim() ?? '';
  };
}

/** Waits for saving to ask `kind`. */
async function question(kind: SavePrompt['kind']): Promise<void> {
  const prompts = TestBed.inject(SavePrompts);
  await vi.waitFor(() => expect(prompts.current()?.kind).toBe(kind));
}

/** Saves the work for the first time, in a project of `store`. */
async function firstSave(store: FakeLibraryStore): Promise<void> {
  const project = store.addProject('Sprites');
  const saving = TestBed.inject(SaveFlow).save();
  await question('project');
  TestBed.inject(SavePrompts).answerProject({ projectId: project.id });
  await saving;
}

describe('the save state pill', () => {
  afterEach(() => document.body.replaceChildren());

  it('says nothing without a document', async () => {
    await configureLibrary();
    const text = await renderPill();

    expect(text()).toBe('');
  });

  it('goes from "Not saved" through "Saving…" to "Saved" only once the write succeeded', async () => {
    const { store } = await configureLibrary();
    await startUnsavedWork();
    const text = await renderPill();
    expect(text()).toBe('Not saved');

    const project = store.addProject('Sprites');
    const saving = TestBed.inject(SaveFlow).save();
    await question('project');
    expect(text()).toBe('Saving…');
    TestBed.inject(SavePrompts).answerProject({ projectId: project.id });
    await saving;

    expect(text()).toBe('Saved');
  });

  it('says "Unsaved changes" once saved work changes', async () => {
    const { store } = await configureLibrary();
    const engine = await startUnsavedWork();
    const text = await renderPill();
    await firstSave(store);

    await engine.apply({ kind: 'setTitle', title: 'Changed' });

    expect(text()).toBe('Unsaved changes');
  });

  it('says "Could not save" after a refused write, never "Saved"', async () => {
    const { store } = await configureLibrary();
    const engine = await startUnsavedWork();
    const text = await renderPill();
    await firstSave(store);
    await engine.apply({ kind: 'setTitle', title: 'Changed' });
    store.failNext('service.unavailable');

    const saving = TestBed.inject(SaveFlow).save();
    await question('failure');
    expect(text()).toBe('Saving…');
    TestBed.inject(SavePrompts).dismiss();
    await saving;

    expect(text()).toBe('Could not save');
    expect(TestBed.inject(EngineStore).document()?.title).toBe('Changed');
  });
});
