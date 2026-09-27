import { inject, Injectable } from '@angular/core';
import { ApiClient } from './api-client';
import type { HttpMethod } from './api-types';
import type { components } from './schema';

/** An account as its owner sees it (accounts.md, H6): reuses H5's `Storage` shape. */
export type Account = components['schemas']['Account'];

/**
 * `ApiClient.request`, without the path typing `schema.d.ts` does not carry yet: H6 builds the
 * server routes below in parallel, so its OpenAPI description and the generated client lag this
 * task (server.md, "OpenAPI and the typed client"). Mirrors the `untyped` helper of
 * `api-client.spec.ts`, written for the same reason.
 */
type UntypedRequest = (
  method: HttpMethod,
  path: string,
  options?: { readonly body?: unknown },
) => Promise<unknown>;

/**
 * H6's account routes (accounts.md, H6 "Types" and "Routes"): the signed-in account, its language,
 * its data export and its deletion. Once `schema.d.ts` describes them, swapping to the generated,
 * typed `ApiClient.request` calls is a one-file change: this file, replacing `untyped` calls with
 * typed ones and dropping the `Account` alias for the generated one.
 */
@Injectable({ providedIn: 'root' })
export class AccountApi {
  private readonly client = inject(ApiClient);
  private readonly untyped = this.client.request.bind(this.client) as unknown as UntypedRequest;

  /**
   * Where a browser downloads the export zip from: cookie-authenticated like any other request,
   * and CSRF-free since it is a `GET`. The account page links to it directly, never through
   * `ApiClient`, which always parses its answer as JSON.
   */
  readonly exportUrl = '/api/v1/account/export';

  get(): Promise<Account> {
    return this.untyped('get', '/api/v1/account') as Promise<Account>;
  }

  updateLanguage(language: string): Promise<Account> {
    return this.untyped('patch', '/api/v1/account', { body: { language } }) as Promise<Account>;
  }

  /** Asks the password again (`auth.current_password` otherwise); ends the session. */
  delete(password: string): Promise<void> {
    return this.untyped('delete', '/api/v1/account', { body: { password } }) as Promise<void>;
  }
}
