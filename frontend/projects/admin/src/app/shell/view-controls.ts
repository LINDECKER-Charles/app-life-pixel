import { ChangeDetectionStrategy, Component, computed, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { SessionStore } from '../core/session-store';
import { DEFAULT_TO, RANGE_PRESETS } from '../core/view-state';
import { ViewStateStore } from '../core/view-state-store';
import { ENVIRONMENT_KEYS } from '../ui/environment-name';

/** The keys of the ranges the selector offers. */
const RANGE_KEYS: Readonly<Record<string, string>> = {
  'now-1h': 'admin.range.last_hour',
  'now-6h': 'admin.range.last_6_hours',
  'now-24h': 'admin.range.last_24_hours',
  'now-7d': 'admin.range.last_7_days',
  'now-30d': 'admin.range.last_30_days',
};

/**
 * The one environment selector and the one time range of every view (admin-console.md): a
 * change rewrites the URL, which every view reads.
 */
@Component({
  selector: 'lp-view-controls',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './view-controls.html',
  styleUrl: './view-controls.scss',
})
export class ViewControls {
  private readonly view = inject(ViewStateStore);

  protected readonly environments = inject(SessionStore).monitoredEnvironments;
  protected readonly state = this.view.state;
  protected readonly rangeKeys = RANGE_KEYS;
  protected readonly environmentKeys = ENVIRONMENT_KEYS;
  /** A range from a shared link that is not a preset stays selectable, under its own name. */
  protected readonly custom = computed(() => {
    const { from, to } = this.state();
    return to === DEFAULT_TO && RANGE_PRESETS.includes(from) ? null : `${from}|${to}`;
  });
  protected readonly presets = RANGE_PRESETS;
  protected readonly selectedRange = computed(() => this.custom() ?? this.state().from);

  protected onEnvironment(event: Event): void {
    void this.view.update({ env: (event.target as HTMLSelectElement).value });
  }

  protected onRange(event: Event): void {
    const value = (event.target as HTMLSelectElement).value;
    if (value !== this.custom()) {
      void this.view.update({ from: value, to: DEFAULT_TO });
    }
  }
}
