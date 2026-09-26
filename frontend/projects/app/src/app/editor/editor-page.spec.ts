import { ComponentRef } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { EngineStore } from '../engine/engine-store';
import { MockEditorEngine } from '../engine/testing/mock-editor-engine';
import { CurrentAnimation } from '../library/current-animation';
import type { FakeLibraryStore } from '../library/testing/fake-library-store';
import { configureLibrary } from '../library/testing/library-test-support';
import { EditorPage } from './editor-page';
import { DiscardConfirmation } from './new-animation/discard-confirmation';
import { NewAnimationFlow } from './new-animation/new-animation-flow';
import { Shortcuts } from './shortcuts';

/** Each region of the layout, with what it holds. */
const REGIONS = {
  '.docbar': ['lp-animation-title', 'lp-save-state-pill', 'lp-save-button', 'lp-export-button'],
  '.rail': ['lp-tool-bar'],
  '.stage': ['lp-canvas'],
  '.inspector': ['lp-palette-panel', 'lp-layer-list', 'lp-playback-preview'],
  '.timeline': ['lp-timeline'],
};

/** The bytes of a document titled `title`, from an engine of its own. */
async function documentTitled(title: string): Promise<Uint8Array> {
  const engine = new MockEditorEngine();
  await engine.create({ title, width: 8, height: 8, layerName: 'Base' });
  return engine.serialize();
}

/** The button whose text is `name`, within `root`. */
function buttonNamed(root: HTMLElement, name: string): HTMLButtonElement {
  const button = Array.from(root.querySelectorAll('button')).find(
    (candidate) => candidate.textContent?.trim() === name,
  );
  if (!button) throw new Error(`no button "${name}"`);
  return button;
}

describe('EditorPage', () => {
  let confirm: ReturnType<typeof vi.fn<() => Promise<boolean>>>;
  let store: FakeLibraryStore;

  async function configure(): Promise<void> {
    confirm = vi.fn<() => Promise<boolean>>();
    ({ store } = await configureLibrary({ provide: DiscardConfirmation, useValue: { confirm } }));
  }

  async function open(animationId?: string): Promise<ComponentFixture<EditorPage>> {
    const fixture = TestBed.createComponent(EditorPage);
    if (animationId !== undefined) {
      (fixture.componentRef as ComponentRef<EditorPage>).setInput('animationId', animationId);
    }
    await fixture.whenStable();
    return fixture;
  }

  afterEach(() => document.body.replaceChildren());

  it('lays out the document bar, the rail, the stage, the inspector and the timeline', async () => {
    await configure();
    const fixture = await open();
    const root = fixture.nativeElement as HTMLElement;

    for (const [region, contents] of Object.entries(REGIONS)) {
      for (const selector of contents) {
        expect(root.querySelector(`${region} ${selector}`), `${region} ${selector}`).not.toBeNull();
      }
    }
  });

  it('welcomes a first visit in the empty stage, without opening the dialog on its own', async () => {
    await configure();
    const fixture = await open();
    const root = fixture.nativeElement as HTMLElement;

    expect(root.querySelector('.stage lp-editor-welcome')).not.toBeNull();
    expect(root.querySelector('.stage lp-canvas')?.hasAttribute('inert')).toBe(true);
    expect(TestBed.inject(NewAnimationFlow).isOpen()).toBe(false);
  });

  it('hides the welcome once an animation is created', async () => {
    await configure();
    const fixture = await open();

    await TestBed.inject(EngineStore).create({ title: 'New', width: 8, height: 8, layerName: 'A' });
    await fixture.whenStable();

    const root = fixture.nativeElement as HTMLElement;
    expect(root.querySelector('lp-editor-welcome')).toBeNull();
    expect(root.querySelector('.stage lp-canvas')?.hasAttribute('inert')).toBe(false);
  });

  it('opens the new-animation form from "Create animation", the title focused', async () => {
    await configure();
    const fixture = await open();

    buttonNamed(fixture.nativeElement, 'Create animation').click();

    await vi.waitFor(() => expect(TestBed.inject(NewAnimationFlow).isOpen()).toBe(true));
    await vi.waitFor(() => expect(document.activeElement?.id).toBe('new-animation-title'));
  });

  it('shows no welcome on a saved animation’s route', async () => {
    await configure();
    const fixture = await open('42');

    expect(fixture.nativeElement.querySelector('lp-editor-welcome')).toBeNull();
    expect(TestBed.inject(NewAnimationFlow).isOpen()).toBe(false);
  });

  it('opens the saved animation its route names', async () => {
    await configure();
    const project = store.addProject('Sprites');
    const saved = store.addAnimation(project.id, 'Saved', await documentTitled('Saved'));

    await open(saved.id);

    const engine = TestBed.inject(EngineStore);
    await vi.waitFor(() => expect(engine.document()?.title).toBe('Saved'));
    expect(TestBed.inject(CurrentAnimation).id()).toBe(saved.id);
  });

  it('holds native New, Save and Export buttons, Export as the primary action', async () => {
    await configure();
    const fixture = await open();
    const docbar = fixture.nativeElement.querySelector('.docbar') as HTMLElement;

    expect(buttonNamed(docbar, 'New').classList).toContain('lp-button--secondary');
    expect(buttonNamed(docbar, 'Save').classList).toContain('lp-button--secondary');
    expect(buttonNamed(docbar, 'Export').classList).toContain('lp-button--primary');
  });

  it('starts a new animation from "New", asking first when there is unsaved work', async () => {
    await configure();
    const engine = TestBed.inject(EngineStore);
    await engine.create({ title: 'Work', width: 8, height: 8, layerName: 'Base' });
    await engine.apply({ kind: 'setTitle', title: 'Work in progress' });
    const fixture = await open();
    confirm.mockResolvedValue(true);

    buttonNamed(fixture.nativeElement, 'New').click();

    await vi.waitFor(() => expect(TestBed.inject(NewAnimationFlow).isOpen()).toBe(true));
    expect(confirm).toHaveBeenCalledOnce();
  });

  it('hands key presses to the shortcuts while it is shown', async () => {
    await configure();
    const calls: string[] = [];
    TestBed.inject(Shortcuts).register([
      { key: 'b', label: 'test.pencil', action: () => calls.push('pencil') },
    ]);
    const fixture = await open();

    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'b' }));
    fixture.componentInstance.ionViewWillLeave();
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'b' }));

    expect(calls).toEqual(['pencil']);
  });
});
