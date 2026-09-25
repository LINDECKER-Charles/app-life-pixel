import { importProvidersFrom } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { provideIonicAngular } from '@ionic/angular';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../i18n/en.json';
import { EditorStore } from '../editor/editor-store';
import { Shortcuts } from '../editor/shortcuts';
import { EngineStore } from '../engine/engine-store';
import { ShortcutsHelpState } from './shortcuts-help-state';
import { ToolBar } from './tool-bar';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };
const NEW_ANIMATION = { title: 'Tools', width: 8, height: 8, layerName: 'Base' };

describe('ToolBar', () => {
  let fixture: ComponentFixture<ToolBar>;
  let engine: EngineStore;

  function handle(event: KeyboardEvent): void {
    TestBed.inject(Shortcuts).handle(event);
  }

  async function setup(): Promise<void> {
    TestBed.configureTestingModule({
      providers: [
        provideIonicAngular({ animated: false }),
        importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING)),
      ],
    });
    await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
    engine = TestBed.inject(EngineStore);
    await engine.create(NEW_ANIMATION);
    document.addEventListener('keydown', handle);
    fixture = TestBed.createComponent(ToolBar);
    await fixture.whenStable();
  }

  function tools(): HTMLButtonElement[] {
    return [...fixture.nativeElement.querySelectorAll('.tool')];
  }

  afterEach(() => {
    document.removeEventListener('keydown', handle);
    document.body.replaceChildren();
  });

  it('renders one aria-pressed button per tool, with the shortcut in its tooltip', async () => {
    await setup();

    const pencil = tools().find((button) => button.getAttribute('aria-pressed') === 'true');
    expect(pencil?.title).toContain('(B)');
    expect(tools()).toHaveLength(7); // six tools plus the rectangle-filled switch
  });

  it('selects a tool on click, updating aria-pressed', async () => {
    await setup();
    const line = tools().find((button) => button.title.includes('(L)'));

    line?.click();
    fixture.detectChanges();

    expect(TestBed.inject(EditorStore).tool()).toBe('line');
    expect(line?.getAttribute('aria-pressed')).toBe('true');
  });

  it('toggles the rectangle filled/outlined switch', async () => {
    await setup();
    const filled = tools().find((button) => button.title.includes('Shift+R'));

    filled?.click();
    fixture.detectChanges();

    expect(TestBed.inject(EditorStore).rectangleFilled()).toBe(true);
    expect(filled?.getAttribute('aria-pressed')).toBe('true');
  });

  it('binds undo and redo to canUndo and canRedo', async () => {
    await setup();
    const [undo, redo] = fixture.nativeElement
      .querySelectorAll('.group')[1]
      .querySelectorAll('button');
    expect(undo.disabled).toBe(true);
    expect(redo.disabled).toBe(true);

    await engine.apply({ kind: 'setTitle', title: 'Changed' });
    fixture.detectChanges();
    expect(undo.disabled).toBe(false);

    undo.click();
    await vi.waitFor(() => expect(engine.state().canRedo).toBe(true));
    fixture.detectChanges();
    expect(redo.disabled).toBe(false);
  });

  it('opens the shortcuts help on ?, and not while typing in a field', async () => {
    await setup();
    const state = TestBed.inject(ShortcutsHelpState);

    document.dispatchEvent(
      new KeyboardEvent('keydown', { key: '?', shiftKey: true, bubbles: true }),
    );
    expect(state.isOpen()).toBe(true);

    state.close();
    const field = document.createElement('input');
    document.body.append(field);
    field.dispatchEvent(new KeyboardEvent('keydown', { key: '?', shiftKey: true, bubbles: true }));
    expect(state.isOpen()).toBe(false);
  });
});
