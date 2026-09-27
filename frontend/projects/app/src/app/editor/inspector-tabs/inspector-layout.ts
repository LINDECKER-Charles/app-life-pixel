import { DestroyRef, DOCUMENT, inject, Injectable, signal } from '@angular/core';

/**
 * Below 75 rem, the inspector shows one panel at a time behind tabs (plan C11,
 * design-system/docs/foundations.md, "Responsive composition"). Kept in step with the literal
 * breakpoints of editor-page.scss.
 */
const TABBED_QUERY = '(max-width: 74.99rem)';

/**
 * Whether the editor is narrower than its persistent layout, following the window as it resizes.
 * Only the inspector's structure depends on it; everything else adapts in CSS alone.
 */
@Injectable({ providedIn: 'root' })
export class InspectorLayout {
  private readonly query = inject(DOCUMENT).defaultView?.matchMedia?.(TABBED_QUERY);
  private readonly matches = signal(this.query?.matches ?? false);

  /** The inspector's panels are tabs rather than stacked sections. */
  readonly tabbed = this.matches.asReadonly();

  constructor() {
    const query = this.query;
    if (!query) return;
    const follow = (event: MediaQueryListEvent): void => this.matches.set(event.matches);
    query.addEventListener('change', follow);
    inject(DestroyRef).onDestroy(() => query.removeEventListener('change', follow));
  }
}
