import { ChangeDetectionStrategy, Component, computed, input } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';

/** The names of the environments the design knows; any other shows as it is configured. */
export const ENVIRONMENT_KEYS: Readonly<Record<string, string>> = {
  local: 'admin.environment.local',
  staging: 'admin.environment.staging',
  production: 'admin.environment.production',
};

/** An environment's name, translated when it is one of the design's. */
@Component({
  selector: 'lp-environment-name',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `@if (key(); as key) {
      {{ key | transloco }}
    } @else {
      {{ environment() }}
    }`,
})
export class EnvironmentName {
  readonly environment = input.required<string>();

  protected readonly key = computed(() => ENVIRONMENT_KEYS[this.environment()] ?? null);
}
