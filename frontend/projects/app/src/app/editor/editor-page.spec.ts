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

const REGION_STUBS = [
  'lp-tool-bar',
  'lp-palette-panel',
  'lp-canvas',
  'lp-timeline',
  'lp-export-button',
];

/** The bytes of a document titled `title`, from an engine of its own. */
async function documentTitled(title: string): Promise<Uint8Array> {
  const engine = new MockEditorEngine();
  await engine.create({ title, width: 8, height: 8, layerName: 'Base' });
  return engine.serialize();
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

  it('holds the five regions the features fill', async () => {
    await configure();
    const fixture = await open();

    for (const selector of REGION_STUBS) {
      expect(fixture.nativeElement.querySelector(selector), selector).not.toBeNull();
    }
  });

  it('asks for a new animation on a first visit', async () => {
    await configure();
    await open();

    expect(TestBed.inject(NewAnimationFlow).isOpen()).toBe(true);
  });

  it('leaves a saved animation to open to its route', async () => {
    await configure();
    await open('42');

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

  it('holds the Save button in its header', async () => {
    await configure();
    const fixture = await open();

    expect(fixture.nativeElement.querySelector('.header lp-save-button ion-button')).not.toBeNull();
  });

  it('starts a new animation from "New", asking first when there is unsaved work', async () => {
    await configure();
    const engine = TestBed.inject(EngineStore);
    await engine.create({ title: 'Work', width: 8, height: 8, layerName: 'Base' });
    await engine.apply({ kind: 'setTitle', title: 'Work in progress' });
    const fixture = await open();
    confirm.mockResolvedValue(true);

    fixture.nativeElement.querySelector('.header ion-button').click();

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
