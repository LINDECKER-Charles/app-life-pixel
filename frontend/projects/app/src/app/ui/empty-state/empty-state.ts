import { booleanAttribute, ChangeDetectionStrategy, Component, input } from '@angular/core';

/** Pip's drawing, copied from design-system/assets/ by tools/copy-design-assets.mjs. */
const PIP_SOURCE = '/design-system/pip.svg';

/**
 * An empty region's explanation (design-system/docs/patterns.md, "Empty states"): why it is
 * empty, what to do, and the action itself, projected. `hasPip` adds the mascot beside the text,
 * decorative and hidden from assistive technology; only the welcome, the library and a project
 * may use it, never an error, a search without results, a deletion or a quota.
 *
 *     <lp-empty-state [heading]="'library.empty.title' | transloco" hasPip>
 *       <a class="lp-button lp-button--primary" routerLink="/editor">…</a>
 *     </lp-empty-state>
 */
@Component({
  selector: 'lp-empty-state',
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    @if (hasPip()) {
      <img class="pip" [src]="pipSource" alt="" width="96" height="96" />
    }
    @if (headingLevel() === 3) {
      <h3 class="heading">{{ heading() }}</h3>
    } @else {
      <h2 class="heading">{{ heading() }}</h2>
    }
    @if (description()) {
      <p class="description">{{ description() }}</p>
    }
    <div class="actions"><ng-content /></div>
  `,
  styles: `
    :host {
      display: flex;
      flex-direction: column;
      gap: var(--lp-space-3);
      align-items: center;
      padding: var(--lp-space-5) var(--lp-space-4);
      text-align: center;
    }
    .pip {
      image-rendering: pixelated;
    }
    .heading {
      margin: 0;
      font-size: var(--lp-font-size-title);
    }
    .description {
      max-width: 40ch;
      margin: 0;
      color: var(--lp-color-text-muted);
    }
    .actions {
      display: flex;
      flex-wrap: wrap;
      gap: var(--lp-space-2);
      justify-content: center;
      margin-top: var(--lp-space-2);

      &:empty {
        display: none;
      }
    }
  `,
})
export class EmptyState {
  /** The translated title: what is absent. */
  readonly heading = input.required<string>();
  /** The translated explanation: how to add it. */
  readonly description = input<string>();
  /** The title's level in the page's outline. */
  readonly headingLevel = input<2 | 3>(2);
  readonly hasPip = input(false, { transform: booleanAttribute });

  protected readonly pipSource = PIP_SOURCE;
}
