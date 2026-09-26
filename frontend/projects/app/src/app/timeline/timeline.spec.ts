import { importProvidersFrom } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { provideIonicAngular } from '@ionic/angular';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../i18n/en.json';
import { EditorStore } from '../editor/editor-store';
import { Shortcuts } from '../editor/shortcuts';
import { EngineStore } from '../engine/engine-store';
import { Timeline } from './timeline';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };
const NEW_ANIMATION = { title: 'Timeline', width: 4, height: 4, layerName: 'Base' };

describe('the timeline', () => {
  afterEach(() => document.body.replaceChildren());

  async function setup() {
    TestBed.configureTestingModule({
      providers: [
        provideIonicAngular({ animated: false }),
        importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING)),
      ],
    });
    await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
    const engine = TestBed.inject(EngineStore);
    const editor = TestBed.inject(EditorStore);
    const shortcuts = TestBed.inject(Shortcuts);
    await engine.create(NEW_ANIMATION);
    const fixture = TestBed.createComponent(Timeline);
    fixture.detectChanges();
    return { fixture, engine, editor, shortcuts };
  }

  it('holds the tags and the frame strip, leaving the layers and the preview to the inspector', async () => {
    const { fixture } = await setup();
    const root = fixture.nativeElement as HTMLElement;

    expect(root.querySelector('lp-tag-bars')).not.toBeNull();
    expect(root.querySelector('lp-frame-list')).not.toBeNull();
    expect(root.querySelector('lp-layer-list')).toBeNull();
    expect(root.querySelector('lp-playback-preview')).toBeNull();
  });

  it('registers `,`, `.` and `O`, unregistered once destroyed', async () => {
    const { fixture, shortcuts } = await setup();

    expect(shortcuts.all().map((shortcut) => shortcut.key)).toEqual(
      expect.arrayContaining([',', '.', 'o']),
    );

    fixture.destroy();

    expect(shortcuts.all().some((shortcut) => [',', '.', 'o'].includes(shortcut.key))).toBe(false);
  });

  it('`,` and `.` step the active frame back and forth', async () => {
    const { engine, editor, shortcuts } = await setup();
    await engine.apply({ kind: 'addFrame', position: 1, durationMs: 100 });
    const [first, second] = engine.document()?.frames.map((frame) => frame.id) ?? [];
    editor.activeFrame.set(first ?? null);

    shortcuts
      .all()
      .find((shortcut) => shortcut.key === '.')
      ?.action();
    expect(editor.activeFrame()).toBe(second);
    expect(editor.frameSelection()).toEqual({ first: 1, last: 1 });

    shortcuts
      .all()
      .find((shortcut) => shortcut.key === ',')
      ?.action();
    expect(editor.activeFrame()).toBe(first);
  });

  it('`O` toggles onion skin', async () => {
    const { editor, shortcuts } = await setup();
    expect(editor.onionSkin().enabled).toBe(false);

    shortcuts
      .all()
      .find((shortcut) => shortcut.key === 'o')
      ?.action();

    expect(editor.onionSkin().enabled).toBe(true);
  });
});
