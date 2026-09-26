import { ChangeDetectionStrategy, Component, computed, input } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { EnvironmentName } from '../ui/environment-name';

/** The banner's colour set: amber for staging, red for production, neutral otherwise. */
export type BannerTone = 'staging' | 'production' | 'neutral';

export function bannerTone(environment: string): BannerTone {
  if (environment === 'staging' || environment === 'production') {
    return environment;
  }
  return 'neutral';
}

/**
 * The permanent banner naming the environment this console administers (admin-console.md): every
 * action of every page changes this environment's data.
 */
@Component({
  selector: 'lp-environment-banner',
  imports: [TranslocoPipe, EnvironmentName],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="banner" [attr.data-tone]="tone()">
      <strong><lp-environment-name [environment]="environment()" /></strong>
      <span>{{ 'admin.banner.actions' | transloco }}</span>
    </div>
  `,
  styles: `
    .banner {
      display: flex;
      flex-wrap: wrap;
      gap: var(--lp-space-2);
      align-items: baseline;
      padding: var(--lp-space-2) var(--lp-space-4);
      font-weight: var(--lp-font-weight-semibold);
      color: var(--lp-color-text);
      background: var(--lp-color-surface);
      border-bottom: var(--lp-border-width-strong) solid var(--lp-color-border);
    }
    strong {
      text-transform: uppercase;
      letter-spacing: 0.05em;
    }
    /* The warning/danger token pairs are measured at 6:1+ in both light and dark mode. */
    [data-tone='staging'] {
      color: var(--lp-color-warning);
      background: var(--lp-color-warning-bg);
      border-color: var(--lp-color-warning);
    }
    [data-tone='production'] {
      color: var(--lp-color-danger);
      background: var(--lp-color-danger-bg);
      border-color: var(--lp-color-danger);
    }
  `,
})
export class EnvironmentBanner {
  readonly environment = input.required<string>();

  protected readonly tone = computed(() => bannerTone(this.environment()));
}
