import { ChangeDetectionStrategy, Component, computed, input } from '@angular/core';
import { ICON_PATHS, type IconName, type IconShape } from './icon-paths';

/** 16 px beside metadata, 20 px in ordinary controls, 24 px in tools and prominent actions. */
export type IconSize = 'small' | 'medium' | 'large';

/**
 * An icon of `ICON_PATHS`, drawn in the text's colour. Always hidden from assistive technology:
 * the control or the text beside it carries the meaning (design-system/docs/foundations.md).
 */
@Component({
  selector: 'lp-icon',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: {
    'aria-hidden': 'true',
    class: 'lp-icon',
    '[class.lp-icon--small]': "size() === 'small'",
    '[class.lp-icon--large]': "size() === 'large'",
  },
  template: `
    <svg viewBox="0 0 24 24" focusable="false">
      <path [attr.d]="shape().outline" />
      @if (shape().solid; as solid) {
        <path class="solid" [attr.d]="solid" />
      }
    </svg>
  `,
  styles: `
    :host {
      display: inline-flex;
      flex-shrink: 0;
      width: var(--lp-icon-size-medium);
      height: var(--lp-icon-size-medium);
    }
    :host(.lp-icon--small) {
      width: var(--lp-icon-size-small);
      height: var(--lp-icon-size-small);
    }
    :host(.lp-icon--large) {
      width: var(--lp-icon-size-large);
      height: var(--lp-icon-size-large);
    }
    svg {
      width: 100%;
      height: 100%;
      fill: none;
      stroke: currentColor;
      stroke-width: 1.75;
      stroke-linecap: round;
      stroke-linejoin: round;
    }
    .solid {
      fill: currentColor;
    }
  `,
})
export class Icon {
  readonly name = input.required<IconName>();
  readonly size = input<IconSize>('medium');

  protected readonly shape = computed((): IconShape => ICON_PATHS[this.name()]);
}
