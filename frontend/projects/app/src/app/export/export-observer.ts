import { InjectionToken } from '@angular/core';
import type { ExportFormat } from '../engine/engine-types';

/**
 * Notified after every download, with the format and its raw size. The web default does nothing;
 * H13 gives the hosted one, which records the product event (editor.md, U5).
 */
export interface ExportObserver {
  record(format: ExportFormat, size: number): void;
}

class NoopExportObserver implements ExportObserver {
  record(): void {
    // No observer by default: H13 provides the hosted one.
  }
}

export const EXPORT_OBSERVER = new InjectionToken<ExportObserver>('EXPORT_OBSERVER', {
  providedIn: 'root',
  factory: () => new NoopExportObserver(),
});
