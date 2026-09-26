import { TestBed } from '@angular/core/testing';
import { Router } from '@angular/router';
import { MockEditorEngine } from '../../engine/testing/mock-editor-engine';
import { CurrentAnimation } from '../current-animation';
import { configureLibrary, startUnsavedWork } from '../testing/library-test-support';
import { OpenAnimationFlow } from './open-animation-flow';
import { OpenConfirmation } from './open-confirmation';

/** The bytes of a document titled `title`, from an engine of its own. */
async function documentTitled(title: string): Promise<Uint8Array> {
  const engine = new MockEditorEngine();
  await engine.create({ title, width: 8, height: 8, layerName: 'Base' });
  return engine.serialize();
}

describe('OpenAnimationFlow', () => {
  let confirm: ReturnType<typeof vi.fn<() => Promise<boolean>>>;

  async function setUp() {
    confirm = vi.fn<() => Promise<boolean>>();
    const bed = await configureLibrary({ provide: OpenConfirmation, useValue: { confirm } });
    const project = bed.store.addProject('Sprites');
    const saved = bed.store.addAnimation(project.id, 'Saved', await documentTitled('Saved'));
    return { ...bed, saved };
  }

  it('opens a saved animation in the editor', async () => {
    const { saved } = await setUp();
    const engine = await startUnsavedWork();
    await engine.markSaved();

    await TestBed.inject(OpenAnimationFlow).open(saved.id);

    expect(engine.document()?.title).toBe('Saved');
    expect(TestBed.inject(CurrentAnimation).state()).toMatchObject({ id: saved.id, version: 1 });
    expect(confirm).not.toHaveBeenCalled();
  });

  it('asks first when there is unsaved work, and keeps it when refused', async () => {
    const { saved } = await setUp();
    const engine = await startUnsavedWork('Mine');
    const router = TestBed.inject(Router);
    await router.navigateByUrl(`/editor/${saved.id}`);
    confirm.mockResolvedValue(false);

    await TestBed.inject(OpenAnimationFlow).open(saved.id);

    expect(confirm).toHaveBeenCalledOnce();
    expect(engine.document()?.title).toBe('Mine');
    expect(engine.hasUnsavedWork()).toBe(true);
    expect(router.url).toBe('/editor');
  });

  it('opens it once the person agrees to lose the unsaved work', async () => {
    const { saved } = await setUp();
    const engine = await startUnsavedWork('Mine');
    confirm.mockResolvedValue(true);

    await TestBed.inject(OpenAnimationFlow).open(saved.id);

    expect(engine.document()?.title).toBe('Saved');
    expect(engine.hasUnsavedWork()).toBe(false);
  });

  it('keeps the work and says why when the animation cannot be read', async () => {
    await setUp();
    const engine = await startUnsavedWork('Mine');
    await engine.markSaved();
    const flow = TestBed.inject(OpenAnimationFlow);

    await flow.open('gone');

    expect(flow.failure()).toEqual({ code: 'library.animation_not_found', params: {} });
    expect(engine.document()?.title).toBe('Mine');
    expect(TestBed.inject(Router).url).toBe('/editor');
  });

  it('sends a visitor to sign in, then back to the animation', async () => {
    const { saved, signedIn } = await setUp();
    signedIn.set(false);

    await TestBed.inject(OpenAnimationFlow).open(saved.id);

    expect(TestBed.inject(Router).url).toBe(`/sign-in?returnUrl=%2Feditor%2F${saved.id}`);
  });

  it('does nothing for the animation the editor already holds', async () => {
    const { saved, store } = await setUp();
    await startUnsavedWork('Mine');
    TestBed.inject(CurrentAnimation).setSaved(saved);
    const openDocument = vi.spyOn(store, 'openDocument');

    await TestBed.inject(OpenAnimationFlow).open(saved.id);

    expect(openDocument).not.toHaveBeenCalled();
    expect(confirm).not.toHaveBeenCalled();
  });
});
