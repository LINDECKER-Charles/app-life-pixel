import { ChangeDetectionStrategy, Component, computed, input } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { Icon } from '../icon/icon';
import type { IconName } from '../icon/icon-paths';

/** The four kinds of message of design-system/docs/foundations.md, "Colour roles". */
export type BannerVariant = 'success' | 'warning' | 'danger' | 'info';

/** Each variant's icon, and the i18n key of its severity in words. */
const VARIANTS: Readonly<Record<BannerVariant, { icon: IconName; severity: string }>> = {
  success: { icon: 'success', severity: 'common.status.success' },
  warning: { icon: 'warning', severity: 'common.status.warning' },
  danger: { icon: 'error', severity: 'common.status.error' },
  info: { icon: 'info', severity: 'common.status.info' },
};

/**
 * A message near the content it concerns (`.lp-banner`): an icon, the severity in words for
 * screen readers, and the projected text; actions go in an element marked `lpBannerActions`.
 * An error is an `alert`, announced at once; the other kinds are a polite `status`. It stays
 * until its owner removes it: never a timer.
 *
 *     <lp-status-banner variant="danger">
 *       <p>{{ 'library.save.failed' | transloco }}</p>
 *       <div lpBannerActions><button class="lp-button lp-button--secondary">…</button></div>
 *     </lp-status-banner>
 */
@Component({
  selector: 'lp-status-banner',
  imports: [Icon, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: {
    class: 'lp-banner',
    '[class.lp-banner--success]': "variant() === 'success'",
    '[class.lp-banner--warning]': "variant() === 'warning'",
    '[class.lp-banner--danger]': "variant() === 'danger'",
    '[class.lp-banner--info]': "variant() === 'info'",
    '[attr.role]': 'role()',
  },
  template: `
    <span class="lp-banner__icon"><lp-icon [name]="details().icon" /></span>
    <div class="lp-banner__body">
      <span class="lp-visually-hidden">{{ details().severity | transloco }}</span>
      <ng-content />
    </div>
    <div class="lp-banner__actions"><ng-content select="[lpBannerActions]" /></div>
  `,
  styles: `
    :host {
      display: grid;
    }
    .lp-banner__actions:empty {
      display: none;
    }
  `,
})
export class StatusBanner {
  readonly variant = input<BannerVariant>('info');

  protected readonly details = computed(() => VARIANTS[this.variant()]);
  protected readonly role = computed(() => (this.variant() === 'danger' ? 'alert' : 'status'));
}
