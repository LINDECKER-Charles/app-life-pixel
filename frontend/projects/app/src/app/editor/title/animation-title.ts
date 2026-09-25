import { ChangeDetectionStrategy, Component, computed, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { EngineStore } from '../../engine/engine-store';

/**
 * The animation's title, editable in place: Enter or leaving the field renames it (`setTitle`),
 * Escape restores it. With no document, it says so.
 */
@Component({
  selector: 'lp-animation-title',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    @if (document(); as document) {
      <label class="lp-visually-hidden" for="animation-title">
        {{ 'editor.title.label' | transloco }}
      </label>
      <input
        #field
        id="animation-title"
        class="title"
        type="text"
        autocomplete="off"
        [value]="document.title"
        [attr.maxlength]="maxLength()"
        (change)="rename(field)"
        (keydown.enter)="field.blur()"
        (keydown.escape)="revert(field)"
      />
    } @else {
      <p class="empty">{{ 'editor.empty.hint' | transloco }}</p>
    }
  `,
  styles: `
    :host {
      display: block;
      min-width: 0;
    }
    .title {
      width: 100%;
      padding: var(--lp-space-1) var(--lp-space-2);
      font: inherit;
      font-size: var(--lp-font-size-large);
      font-weight: var(--lp-font-weight-bold);
      color: var(--lp-color-text);
      background: transparent;
      border: 1px solid transparent;
      border-radius: var(--lp-radius-small);
    }
    .title:hover,
    .title:focus {
      border-color: var(--lp-color-border);
    }
    .empty {
      margin: 0;
      color: var(--lp-color-text-muted);
    }
  `,
})
export class AnimationTitle {
  private readonly engine = inject(EngineStore);

  protected readonly document = this.engine.document;
  protected readonly maxLength = computed(() => this.engine.limits()?.nameMaxChars ?? null);

  /** Renames the animation; an empty or rejected title gives way to the document's. */
  protected async rename(field: HTMLInputElement): Promise<void> {
    const title = field.value.trim();
    if (title.length > 0 && title !== this.document()?.title) {
      await this.engine.apply({ kind: 'setTitle', title });
    }
    field.value = this.document()?.title ?? '';
  }

  protected revert(field: HTMLInputElement): void {
    field.value = this.document()?.title ?? '';
    field.blur();
  }
}
