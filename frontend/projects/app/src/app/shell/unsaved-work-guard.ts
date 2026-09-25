import {
  DOCUMENT,
  effect,
  EnvironmentProviders,
  inject,
  makeEnvironmentProviders,
  provideEnvironmentInitializer,
} from '@angular/core';
import { EngineStore } from '../engine/engine-store';

/** Makes the browser ask before the page is left or reloaded. */
function askBeforeLeaving(event: BeforeUnloadEvent): void {
  event.preventDefault();
  // Browsers that predate `preventDefault` on this event ask when `returnValue` is set.
  event.returnValue = '';
}

function guardUnsavedWork(): void {
  const engine = inject(EngineStore);
  const view = inject(DOCUMENT).defaultView;
  if (!view) return;
  effect((onCleanup) => {
    if (!engine.hasUnsavedWork()) return;
    view.addEventListener('beforeunload', askBeforeLeaving);
    onCleanup(() => view.removeEventListener('beforeunload', askBeforeLeaving));
  });
}

/**
 * While the page holds unsaved work, the browser asks before it is left or reloaded: nothing is
 * kept in the browser, so leaving loses the work (D37). The handler is set only meanwhile.
 */
export function provideUnsavedWorkGuard(): EnvironmentProviders {
  return makeEnvironmentProviders([provideEnvironmentInitializer(guardUnsavedWork)]);
}
