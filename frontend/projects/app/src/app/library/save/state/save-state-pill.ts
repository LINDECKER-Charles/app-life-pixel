import { ChangeDetectionStrategy, Component, computed, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { EngineStore } from '../../../engine/engine-store';
import { Icon } from '../../../ui/icon/icon';
import type { IconName } from '../../../ui/icon/icon-paths';
import { CurrentAnimation } from '../../current-animation';
import { SaveFlow } from '../save-flow';
import { saveState, type SaveState } from './save-state';

/** How each state looks: its words, and an icon — or a dot — so colour never carries it alone. */
interface SaveStateLook {
  readonly label: string;
  readonly variant: string;
  readonly icon: IconName | null;
}

const LOOKS: Readonly<Record<SaveState, SaveStateLook>> = {
  'not-saved': { label: 'library.save.state.not_saved', variant: 'warning', icon: 'warning' },
  'unsaved-changes': { label: 'library.save.state.unsaved', variant: 'accent', icon: null },
  saving: { label: 'library.save.saving', variant: 'info', icon: null },
  saved: { label: 'library.save.saved', variant: 'success', icon: 'check' },
  failed: { label: 'library.save.state.failed', variant: 'danger', icon: 'error' },
};

/**
 * The document bar's save state (C9): one pill beside the title, in a live region that announces
 * each change once. Its text is the state alone, so that "Saved" reads as exactly that.
 */
@Component({
  selector: 'lp-save-state-pill',
  imports: [Icon, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <span role="status">
      @if (look(); as look) {
        <span class="lp-pill lp-pill--{{ look.variant }}">
          @if (look.icon; as icon) {
            <lp-icon [name]="icon" size="small" />
          } @else {
            <span class="lp-pill__dot"></span>
          }
          {{ look.label | transloco }}
        </span>
      }
    </span>
  `,
  styles: `
    :host {
      display: inline-flex;
      min-width: 0;
    }
  `,
})
export class SaveStatePill {
  private readonly flow = inject(SaveFlow);
  private readonly engine = inject(EngineStore);
  private readonly current = inject(CurrentAnimation);

  protected readonly look = computed(() => {
    const state = saveState({
      status: this.flow.status(),
      hasDocument: this.engine.document() !== null,
      hasUnsavedWork: this.engine.hasUnsavedWork(),
      current: this.current.state().kind,
    });
    return state === null ? null : LOOKS[state];
  });
}
