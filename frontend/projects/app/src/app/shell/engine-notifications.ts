import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { EngineStore } from '../engine/engine-store';

/** The engine's refusals, translated from their `errors.<code>` key, until dismissed. */
@Component({
  selector: 'lp-engine-notifications',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <ul class="notifications" aria-live="polite">
      @for (notification of engine.notifications(); track notification.id) {
        <li class="notification">
          <span>{{ notification.key | transloco: notification.params }}</span>
          <button type="button" (click)="engine.dismissNotification(notification.id)">
            {{ 'common.close' | transloco }}
          </button>
        </li>
      }
    </ul>
  `,
  styles: `
    .notifications {
      position: fixed;
      inset-block-end: var(--lp-space-4);
      inset-inline: var(--lp-space-4);
      z-index: 30000;
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: var(--lp-space-2);
      padding: 0;
      margin: 0;
      list-style: none;
      pointer-events: none;
    }
    .notification {
      display: flex;
      align-items: center;
      gap: var(--lp-space-3);
      max-width: 40rem;
      padding: var(--lp-space-2) var(--lp-space-3);
      color: var(--lp-color-on-danger);
      background: var(--lp-color-danger);
      border-radius: var(--lp-radius-medium);
      pointer-events: auto;
    }
    button {
      padding: var(--lp-space-1) var(--lp-space-2);
      font: inherit;
      color: inherit;
      background: transparent;
      border: 1px solid currentColor;
      border-radius: var(--lp-radius-small);
      cursor: pointer;
    }
  `,
})
export class EngineNotifications {
  protected readonly engine = inject(EngineStore);
}
