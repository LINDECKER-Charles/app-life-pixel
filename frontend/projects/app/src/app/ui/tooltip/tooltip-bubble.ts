import { ChangeDetectionStrategy, Component, input } from '@angular/core';

/** Which side of its control a tooltip opens on. */
export type TooltipPlacement = 'right' | 'top';

/**
 * The bubble a `lpTooltip` control shows: the control's name and, as a key cap, its shortcut.
 * Fixed to the page body, so that no scrolling panel clips it; `Tooltip` places it.
 */
@Component({
  selector: 'lp-tooltip-bubble',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: {
    role: 'tooltip',
    '[id]': 'id()',
    '[class]': "'placement-' + placement()",
    '[style.left.px]': 'left()',
    '[style.top.px]': 'top()',
  },
  template: `
    <span>{{ text() }}</span>
    @if (shortcut()) {
      <kbd class="key">{{ shortcut() }}</kbd>
    }
  `,
  styles: `
    :host {
      position: fixed;
      z-index: var(--lp-z-tooltip);
      display: flex;
      gap: var(--lp-space-2);
      align-items: center;
      max-width: min(20rem, calc(100vw - 2 * var(--lp-space-3)));
      padding: var(--lp-space-1) var(--lp-space-2);
      font-size: var(--lp-font-size-small);
      font-weight: var(--lp-font-weight-bold);
      line-height: var(--lp-font-line-height-control);
      color: var(--lp-color-background);
      background: var(--lp-color-text);
      border: var(--lp-border-width) solid transparent;
      border-radius: var(--lp-radius-medium);
      box-shadow: var(--lp-shadow-medium);
    }
    :host(.placement-right) {
      transform: translateY(-50%);
    }
    :host(.placement-top) {
      transform: translate(-50%, -100%);
    }
    .key {
      padding: 0 var(--lp-space-1);
      font-family: var(--lp-font-family-mono);
      font-weight: var(--lp-font-weight-regular);
      border: var(--lp-border-width) solid currentColor;
      border-radius: var(--lp-radius-small);
    }
  `,
})
export class TooltipBubble {
  readonly id = input.required<string>();
  readonly text = input.required<string>();
  readonly shortcut = input('');
  readonly placement = input<TooltipPlacement>('right');
  readonly left = input(0);
  readonly top = input(0);
}
