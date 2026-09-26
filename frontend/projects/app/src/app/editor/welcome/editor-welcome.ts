import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { LIBRARY_ACCESS } from '../../library/library-access';
import { EmptyState } from '../../ui/empty-state/empty-state';
import { Icon } from '../../ui/icon/icon';
import { NewAnimationFlow } from '../new-animation/new-animation-flow';

/**
 * The first screen of an empty editor (plan C10, design-system/docs/journeys.md §1): what can be
 * made, Pip, and one action, "Create animation", which opens the new-animation dialog. A visitor
 * also reads that nothing needs an account, and that unsaved work does not outlive the page (D37).
 */
@Component({
  selector: 'lp-editor-welcome',
  imports: [EmptyState, Icon, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <lp-empty-state
      class="lp-card"
      hasPip
      [heading]="'editor.welcome.title' | transloco"
      [description]="'editor.welcome.description' | transloco"
    >
      <button type="button" class="lp-button lp-button--primary" (click)="flow.start()">
        <lp-icon name="plus" />
        {{ 'editor.welcome.create' | transloco }}
      </button>
      @if (!access.signedIn()) {
        <div class="notice">
          <lp-icon name="info" size="small" />
          <p>
            {{ 'editor.welcome.guest_notice' | transloco }}
            {{ 'editor.welcome.guest_loss' | transloco }}
          </p>
        </div>
      }
    </lp-empty-state>
  `,
  styles: `
    :host {
      display: grid;
      // Centred while it fits; when the stage is short, it starts at the top and scrolls.
      place-items: safe center;
      padding: var(--lp-space-3);
      overflow-y: auto;
    }
    lp-empty-state {
      max-width: 36rem;
      // Tighter than an empty region's, so that it fits the stage of a 768-pixel-high window.
      padding: var(--lp-space-4);
      box-shadow: var(--lp-shadow-medium);
    }
    .notice {
      display: flex;
      flex-basis: 100%;
      gap: var(--lp-space-2);
      align-items: flex-start;
      font-size: var(--lp-font-size-small);
      color: var(--lp-color-text-muted);
      text-align: start;

      p {
        margin: 0;
      }
    }
  `,
})
export class EditorWelcome {
  protected readonly flow = inject(NewAnimationFlow);
  protected readonly access = inject(LIBRARY_ACCESS);
}
