import { TestBed } from '@angular/core/testing';
import { emit } from '@tauri-apps/api/event';
import { mockIPC } from '@tauri-apps/api/mocks';
import { EngineStore } from '../../engine/engine-store';
import { MockEditorEngine } from '../../engine/testing/mock-editor-engine';
import { LibraryChangedPrompt } from '../../library/changes/library-changed-prompt';
import { type LibraryChange, LibraryChanges } from '../../library/changes/library-changes';
import { AnimationLoader } from '../../library/open/animation-loader';
import { configureLibrary } from '../../library/testing/library-test-support';
import { clearTauriMocks } from '../testing/clear-tauri-mocks';
import { DesktopLibraryEvents, LIBRARY_CHANGED_EVENT } from './desktop-library-events';

async function documentTitled(title: string): Promise<Uint8Array> {
  const engine = new MockEditorEngine();
  await engine.create({ title, width: 8, height: 8, layerName: 'Base' });
  return engine.serialize();
}

describe('DesktopLibraryEvents', () => {
  let ask: ReturnType<typeof vi.fn>;

  /** The animation "Saved" open in the editor, then written by an agent, with events mocked. */
  async function setUp({ unsavedWork }: { unsavedWork: boolean }) {
    mockIPC(() => null, { shouldMockEvents: true });
    ask = vi.fn().mockResolvedValue('keep');
    const { store } = await configureLibrary({ provide: LibraryChangedPrompt, useValue: { ask } });
    const project = store.addProject('Sprites');
    const saved = store.addAnimation(project.id, 'Saved', await documentTitled('Saved'));
    await TestBed.inject(AnimationLoader).load(saved.id);
    const engine = TestBed.inject(EngineStore);
    if (unsavedWork) await engine.apply({ kind: 'setTitle', title: 'Mine' });
    await store.saveDocument(saved.id, await documentTitled('By agent'), 1);
    await TestBed.inject(DesktopLibraryEvents).follow();
    const change: LibraryChange = { projectIds: [project.id], animationIds: [saved.id] };
    return { engine, change };
  }

  afterEach(() => clearTauriMocks());

  it('passes each library-changed event to the library’s followers', async () => {
    const { change } = await setUp({ unsavedWork: false });
    const received: LibraryChange[] = [];
    TestBed.inject(LibraryChanges).changes.subscribe((each) => received.push(each));

    await emit(LIBRARY_CHANGED_EVENT, change);

    expect(received).toEqual([change]);
  });

  it('reloads the open animation changed on disk when it holds no unsaved work', async () => {
    const { engine, change } = await setUp({ unsavedWork: false });

    await emit(LIBRARY_CHANGED_EVENT, change);

    await vi.waitFor(() => expect(engine.document()?.title).toBe('By agent'));
    expect(ask).not.toHaveBeenCalled();
  });

  it('asks before touching unsaved work, which keeping leaves as it is', async () => {
    const { engine, change } = await setUp({ unsavedWork: true });

    await emit(LIBRARY_CHANGED_EVENT, change);

    await vi.waitFor(() => expect(ask).toHaveBeenCalledOnce());
    expect(engine.document()?.title).toBe('Mine');
    expect(engine.hasUnsavedWork()).toBe(true);
  });
});
