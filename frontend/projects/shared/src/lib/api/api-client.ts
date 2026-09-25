import { HttpClient, HttpParams } from '@angular/common/http';
import { inject, Injectable } from '@angular/core';
import { firstValueFrom } from 'rxjs';
import { API_HEADERS, LIFE_PIXEL_CLIENT, LIFE_PIXEL_CLIENT_HEADER } from './api-headers';
import { ApiProblem } from './api-problem';
import { HttpMethod, Operation, OptionsArgument, PathsWith, ResponseOf } from './api-types';

type QueryValue = string | number | boolean | readonly (string | number | boolean)[];

/** A request before its types are checked away. */
interface RawOptions {
  readonly path?: Readonly<Record<string, string | number>>;
  readonly query?: Readonly<Record<string, QueryValue | undefined>>;
  readonly body?: unknown;
}

/** `/projects/{projectId}` with `{ projectId: 'a b' }`: `/projects/a%20b`. */
function expandPath(template: string, parameters: RawOptions['path'] = {}): string {
  return template.replace(/\{(\w+)\}/g, (_, name: string) =>
    encodeURIComponent(String(parameters[name])),
  );
}

function toParams(query: RawOptions['query'] = {}): HttpParams {
  const defined = Object.entries(query).filter(
    (entry): entry is [string, QueryValue] => entry[1] !== undefined,
  );
  return new HttpParams({ fromObject: Object.fromEntries(defined) });
}

/**
 * The API, typed by path and method from `schema.d.ts`: the only user of `HttpClient`. It sends
 * `Life-Pixel-Client` and the headers of `API_HEADERS`, and throws an `ApiProblem` for any
 * failure. Each feature adds a service beside it.
 */
@Injectable({ providedIn: 'root' })
export class ApiClient {
  private readonly http = inject(HttpClient);
  private readonly client = inject(LIFE_PIXEL_CLIENT);
  private readonly headerSources = inject(API_HEADERS, { optional: true }) ?? [];

  /** Sends the operation of `method` on `path`, and resolves to its JSON answer. */
  async request<M extends HttpMethod, P extends PathsWith<M>>(
    method: M,
    path: P,
    ...[options]: OptionsArgument<Operation<P, M>>
  ): Promise<ResponseOf<Operation<P, M>>> {
    const raw = (options ?? {}) as RawOptions;
    const answer = await this.send(method, expandPath(path, raw.path), raw);
    return (answer ?? undefined) as ResponseOf<Operation<P, M>>;
  }

  private async send(method: HttpMethod, url: string, options: RawOptions): Promise<unknown> {
    const headers = this.headersFor(method, url);
    const request = this.http.request(method.toUpperCase(), url, {
      body: options.body,
      headers,
      params: toParams(options.query),
      observe: 'body',
      responseType: 'json',
    });
    try {
      return await firstValueFrom(request);
    } catch (error: unknown) {
      throw ApiProblem.from(error);
    }
  }

  private headersFor(method: HttpMethod, url: string): Record<string, string> {
    return this.headerSources.reduce<Record<string, string>>(
      (headers, source) => ({ ...headers, ...source({ method, url }) }),
      { [LIFE_PIXEL_CLIENT_HEADER]: this.client },
    );
  }
}
