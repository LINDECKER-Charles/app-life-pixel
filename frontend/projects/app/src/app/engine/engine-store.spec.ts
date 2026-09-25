import { TestBed } from '@angular/core/testing';
import { EDITOR_ENGINE } from './editor-engine';
import type { EngineRecovery, RecoveryReport } from './engine-recovery';
import { EngineStore } from './engine-store';
import { MockEditorEngine } from './testing/mock-editor-engine';

const NEW_ANIMATION = { title: 'Store', width: 4, height: 4, layerName: 'Base' };

/** The mock, with the recovery of an engine whose worker can fail. */
class RecoveringEngine extends MockEditorEngine implements EngineRecovery {
  private readonly recoveryListeners = new Set<(report: RecoveryReport) => void>();

  onRecovered(listener: (report: RecoveryReport) => void): () => void {
    this.recoveryListeners.add(listener);
    return () => this.recoveryListeners.delete(listener);
  }

  recover(report: RecoveryReport): void {
    for (const listener of this.recoveryListeners) listener(report);
  }
}

describe('EngineStore', () => {
  function setup(): EngineStore {
    TestBed.configureTestingModule({});
    return TestBed.inject(EngineStore);
  }

  it('publishes the engine state and its computed signals', async () => {
    const store = setup();
    expect(store.state().status).toBe('empty');

    await store.create(NEW_ANIMATION);

    expect(store.document()?.title).toBe(NEW_ANIMATION.title);
    expect(store.limits()?.canvasMaxSide).toBe(512);
    expect(store.hasUnsavedWork()).toBe(false);
  });

  it('turns a rejected command into a notification keyed by its error code', async () => {
    const store = setup();
    await store.create(NEW_ANIMATION);

    await store.apply({ kind: 'deleteLayer', layer: 9_999 });

    expect(store.notifications()).toHaveLength(1);
    expect(store.notifications()[0].key).toBe('errors.edit.layer_not_found');
  });

  it('dismisses a notification', async () => {
    const store = setup();
    await store.create(NEW_ANIMATION);
    await store.apply({ kind: 'deleteLayer', layer: 9_999 });
    const [notification] = store.notifications();

    store.dismissNotification(notification.id);

    expect(store.notifications()).toHaveLength(0);
  });

  it('says how many changes an engine that recovered lost', () => {
    const engine = new RecoveringEngine();
    TestBed.configureTestingModule({ providers: [{ provide: EDITOR_ENGINE, useValue: engine }] });
    const store = TestBed.inject(EngineStore);

    engine.recover({ lostChanges: 3 });

    expect(store.notifications()).toEqual([
      { id: 1, key: 'engine.recovered', params: { lostChanges: 3 } },
    ]);
  });
});
