import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { EngineStore } from '../engine/engine-store';
import { Icon } from '../ui/icon/icon';

/**
 * The engine's refusals, translated from their `errors.<code>` key, until dismissed: persistent
 * errors, never on a timer (design-system/docs/components.md). They take their own row under the
 * header, so they never cover the canvas, a tool or an action row; on the toast layer, they stay
 * visible above an open dialog's scrim.
 */
@Component({
  selector: 'lp-engine-notifications',
  imports: [Icon, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <ul
      class="notifications"
      aria-live="polite"
      [attr.aria-label]="'shell.notifications.label' | transloco"
    >
      @for (notification of engine.notifications(); track notification.id) {
        <li class="notification">
          <lp-icon name="error" />
          <span class="message">
            <span class="lp-visually-hidden">{{ 'common.status.error' | transloco }}</span>
            {{ notification.key | transloco: notification.params }}
          </span>
          <button
            type="button"
            class="lp-icon-button lp-button--compact dismiss"
            [attr.aria-label]="'common.dismiss' | transloco"
            (click)="engine.dismissNotification(notification.id)"
          >
            <lp-icon name="close" />
          </button>
        </li>
      }
    </ul>
  `,
  styles: `
    .notifications {
      position: relative;
      z-index: var(--lp-z-toast);
      display: grid;
      gap: var(--lp-space-2);
      padding: 0;
      margin: 0;
      list-style: none;

      &:not(:empty) {
        padding: var(--lp-space-2) var(--lp-space-4);
      }
    }
    .notification {
      box-sizing: border-box;
      display: flex;
      gap: var(--lp-space-3);
      align-items: center;
      width: 100%;
      max-width: var(--lp-layout-reading);
      padding: var(--lp-space-1) var(--lp-space-1) var(--lp-space-1) var(--lp-space-3);
      margin-inline: auto;
      font-size: var(--lp-font-size-small);
      font-weight: var(--lp-font-weight-semibold);
      color: var(--lp-color-danger);
      background: var(--lp-color-danger-bg);
      border: var(--lp-border-width) solid var(--lp-color-danger);
      border-radius: var(--lp-radius-large);
    }
    .message {
      flex: 1;
      min-width: 0;
    }
    .dismiss {
      color: inherit;
    }
  `,
})
export class EngineNotifications {
  protected readonly engine = inject(EngineStore);
}
