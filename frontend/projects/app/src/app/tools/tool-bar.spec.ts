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
    document.body.append(fixture.nativeElement); // so that its buttons can take the focus
    await fixture.whenStable();
  }

  function button(name: string): HTMLButtonElement {
    const found = [...fixture.nativeElement.querySelectorAll('button')].find(
      (candidate: HTMLButtonElement) => candidate.getAttribute('aria-label') === name,
    );
    if (!found) throw new Error(`No button named ${name}`);
    return found;
  }

  function pressed(): string[] {
    return [...fixture.nativeElement.querySelectorAll('[aria-pressed="true"]')].map(
      (element: Element) => element.getAttribute('aria-label') ?? '',
    );
  }

  function tooltipOf(control: HTMLElement): HTMLElement | null {
    const id = control.getAttribute('aria-describedby');
    return id ? document.getElementById(id) : null;
  }

  /** Focuses a control as Tab does, which makes it `:focus-visible` to jsdom too. */
  function tabTo(control: HTMLElement): void {
    document.body.addEventListener('keydown', () => control.focus(), { once: true });
    document.body.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true }));
  }

  afterEach(() => {
    document.removeEventListener('keydown', handle);
    fixture.destroy();
    document.body.replaceChildren();
  });

  it('names each icon button by its tool, the pencil pressed first', async () => {
    await setup();

    for (const name of ['Pencil', 'Eraser', 'Fill', 'Line', 'Rectangle', 'Select and move']) {
      expect(button(name).querySelector('lp-icon')).not.toBeNull();
    }
    expect(pressed()).toEqual(['Pencil']);
  });

  it('exposes the selected tool by aria-pressed, from a click or a shortcut', async () => {
    await setup();

    button('Line').click();
    fixture.detectChanges();
    expect(TestBed.inject(EditorStore).tool()).toBe('line');
    expect(pressed()).toEqual(['Line']);

    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'm', bubbles: true }));
    fixture.detectChanges();
    expect(pressed()).toEqual(['Select and move']);
  });

  it('shows a tooltip with the shortcut on keyboard focus and on hover', async () => {
    await setup();
    const filled = button('Filled rectangle');
    const line = button('Line');

    tabTo(filled);
    expect(tooltipOf(filled)?.textContent).toContain('Filled rectangle');
    expect(tooltipOf(filled)?.textContent).toContain('Shift+R');
    filled.blur();

    line.dispatchEvent(new PointerEvent('pointerenter'));
    expect(tooltipOf(line)?.textContent).toContain('Line');
    expect(tooltipOf(line)?.querySelector('kbd')?.textContent).toBe('L');
    line.dispatchEvent(new PointerEvent('pointerdown'));
    expect(tooltipOf(line)).toBeNull();
  });

  it('toggles the filled switch beside the rectangle', async () => {
    await setup();
    const rectangle = button('Rectangle');
    const filled = button('Filled rectangle');

    filled.click();
    fixture.detectChanges();

    expect(rectangle.nextElementSibling).toBe(filled);
    expect(TestBed.inject(EditorStore).rectangleFilled()).toBe(true);
    expect(filled.getAttribute('aria-pressed')).toBe('true');
  });

  it('binds undo and redo, side by side, to canUndo and canRedo', async () => {
    await setup();
    const undo = button('Undo');
    const redo = button('Redo');
    expect(undo.nextElementSibling).toBe(redo);
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
