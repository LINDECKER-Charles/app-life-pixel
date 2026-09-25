import { InjectionToken } from '@angular/core';
import { HttpMethod } from './api-types';

/** The web app's version, raised with each release. */
const WEB_APP_VERSION = '0.0.0';

/** `Life-Pixel-Client`: the platform and version the server checks against its minimums. */
export const LIFE_PIXEL_CLIENT_HEADER = 'Life-Pixel-Client';

/** `<platform>/<version>`: the web app's by default; the desktop app provides its own. */
export const LIFE_PIXEL_CLIENT = new InjectionToken<string>('LIFE_PIXEL_CLIENT', {
  providedIn: 'root',
  factory: () => `web/${WEB_APP_VERSION}`,
});

/** A request on its way, as a header source sees it. */
export interface ApiRequestInfo {
  readonly method: HttpMethod;
  readonly url: string;
}

/** Returns the headers a request must carry, such as the session's CSRF token. */
export type ApiHeaderSource = (request: ApiRequestInfo) => Readonly<Record<string, string>>;

/**
 * Headers every API request gets, from each source in the order provided: H5 adds the CSRF
 * header this way, `{ provide: API_HEADERS, useFactory: …, multi: true }`.
 */
export const API_HEADERS = new InjectionToken<readonly ApiHeaderSource[]>('API_HEADERS');
