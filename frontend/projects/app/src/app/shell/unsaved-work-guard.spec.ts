import { TestBed } from '@angular/core/testing';
import { EngineStore } from '../engine/engine-store';
import { provideUnsavedWorkGuard } from './unsaved-work-guard';

const NEW_ANIMATION = { title: 'Guarded', width: 8, height: 8, layerName: 'Base' };

/** Whether leaving the page now would make the browser ask first. */
function leavingAsks(): boolean {
  const event = new Event('beforeunload', { cancelable: true });
  window.dispatchEvent(event);
  return event.defaultPrevented;
}

describe('provideUnsavedWorkGuard', () => {
  function setup(): EngineStore {
    TestBed.configureTestingModule({ providers: [provideUnsavedWorkGuard()] });
    return TestBed.inject(EngineStore);
  }

  it('asks before the page is left only while there is unsaved work', async () => {
    const engine = setup();
    TestBed.tick();
    expect(leavingAsks()).toBe(false);

    await engine.create(NEW_ANIMATION);
    TestBed.tick();
    expect(leavingAsks()).toBe(false);

    await engine.apply({ kind: 'addLayer', position: 1, name: 'Top' });
    TestBed.tick();
    expect(leavingAsks()).toBe(true);

    await engine.markSaved();
    TestBed.tick();
    expect(leavingAsks()).toBe(false);
  });

  it('stops asking once the work is undone', async () => {
    const engine = setup();
    await engine.create(NEW_ANIMATION);
    await engine.apply({ kind: 'setTitle', title: 'Renamed' });
    TestBed.tick();
    expect(leavingAsks()).toBe(true);

    await engine.undo();
    TestBed.tick();

    expect(leavingAsks()).toBe(false);
  });
});
