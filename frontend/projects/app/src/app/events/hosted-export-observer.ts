import { inject, Injectable } from '@angular/core';
import { TranslocoService } from '@jsverse/transloco';
import { EventsApi, LIFE_PIXEL_CLIENT } from 'shared';
import type { ExportFormat } from '../engine/engine-types';
import { ExportObserver } from '../export/export-observer';

/**
 * The web's `EXPORT_OBSERVER` (H13): reports every download as an `export_completed` product
 * event. Provided on the web only; the desktop app keeps U5's silent default.
 */
@Injectable({ providedIn: 'root' })
export class HostedExportObserver implements ExportObserver {
  private readonly events = inject(EventsApi);
  private readonly client = inject(LIFE_PIXEL_CLIENT);
  private readonly transloco = inject(TranslocoService);

  record(format: ExportFormat, size: number): void {
    const [platform, appVersion] = this.client.split('/');
    void this.events.exportCompleted(format, size, {
      platform,
      appVersion,
      language: this.transloco.getActiveLang(),
    });
  }
}
