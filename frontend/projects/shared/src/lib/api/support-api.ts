import { inject, Injectable } from '@angular/core';
import { ApiClient } from './api-client';
import type { HttpMethod } from './api-types';
import type { components } from './schema';

/** What a support request is about (support-admin.md, H9). */
export type SupportCategory = components['schemas']['SupportCategory'];
/** Where a support request stands. */
export type SupportRequestStatus = components['schemas']['SupportRequestStatus'];
/** A message of a request, as its author's account sees it: never an internal note. */
export type SupportRequestMessage = components['schemas']['SupportRequestMessage'];
/** A request, without its messages. */
export type SupportRequestSummary = components['schemas']['SupportRequestSummary'];
/** A request and its messages, oldest first: the first is the request's own. */
export type SupportRequestThread = components['schemas']['SupportRequestThread'];
/** A page of requests, the most recently updated first. */
export type SupportRequestPage = components['schemas']['Page_SupportRequestSummary'];
/** What the app attaches to a request without asking: `screen` is a route template. */
export type SupportRequestContext = components['schemas']['SupportRequestContext'];

/** A new request, as the form sends it. */
export interface NewSupportRequest {
  readonly category: SupportCategory;
  readonly message: string;
  readonly context: SupportRequestContext;
  readonly screenshot?: Blob;
}

/**
 * `ApiClient.request` with a body its typing cannot describe: the creation is a
 * `multipart/form-data` form, and the generated types only carry JSON bodies.
 */
type FormRequest = (
  method: HttpMethod,
  path: string,
  options: { readonly body: FormData },
) => Promise<unknown>;

/**
 * H9's support routes (support-admin.md, H9): a signed-in account's requests to the team, their
 * threads, and its replies. The screenshot is never served back.
 */
@Injectable({ providedIn: 'root' })
export class SupportApi {
  private readonly client = inject(ApiClient);
  private readonly form = this.client.request.bind(this.client) as unknown as FormRequest;

  /** Sends `request` as a form: the browser sets its boundary. */
  create(request: NewSupportRequest): Promise<SupportRequestThread> {
    const body = new FormData();
    body.append('category', request.category);
    body.append('message', request.message);
    body.append('context', JSON.stringify(request.context));
    if (request.screenshot) {
      body.append('screenshot', request.screenshot, 'screenshot');
    }
    return this.form('post', '/api/v1/support-requests', {
      body,
    }) as Promise<SupportRequestThread>;
  }

  /** The page after `cursor`, or the first. */
  list(cursor?: string): Promise<SupportRequestPage> {
    return this.client.request('get', '/api/v1/support-requests', { query: { cursor } });
  }

  get(requestId: string): Promise<SupportRequestThread> {
    return this.client.request('get', '/api/v1/support-requests/{id}', {
      path: { id: requestId },
    });
  }

  /** Adds `body` to the request: `support.request_closed` once it is closed. */
  reply(requestId: string, body: string): Promise<SupportRequestMessage> {
    return this.client.request('post', '/api/v1/support-requests/{id}/messages', {
      path: { id: requestId },
      body: { body },
    });
  }
}
