import { TestBed } from '@angular/core/testing';
import { EngineStore } from './engine-store';

const NEW_ANIMATION = { title: 'Store', width: 4, height: 4, layerName: 'Base' };

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
});
