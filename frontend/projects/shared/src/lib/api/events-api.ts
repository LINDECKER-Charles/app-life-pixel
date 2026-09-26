import { inject, Injectable } from '@angular/core';
import { ApiClient } from './api-client';
import { components } from './schema';

/** The format a completed export used, as the server's allow-list fixes the list. */
export type ExportFormat = components['schemas']['ExportFormat'];

/** The app's browsing context every product event carries (H13). */
export interface EventContext {
  readonly platform: string;
  readonly appVersion: string;
  readonly language: string;
}

/**
 * `POST /api/v1/events`: the app's allow-list of product events (H13). Never throws; a failed or
 * dropped event is not the caller's problem.
 */
@Injectable({ providedIn: 'root' })
export class EventsApi {
  private readonly client = inject(ApiClient);

  /** Reports that an export of `format` finished, its raw size in bytes. */
  async exportCompleted(format: ExportFormat, bytes: number, context: EventContext): Promise<void> {
    await this.client.request('post', '/api/v1/events', {
      body: {
        name: 'export_completed',
        properties: { format, bytes },
        platform: context.platform,
        appVersion: context.appVersion,
        language: context.language,
      },
    });
  }
}
