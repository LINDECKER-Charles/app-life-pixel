import {
  HttpClient,
  HttpErrorResponse,
  HttpHeaders,
  HttpParams,
  HttpResponse,
} from '@angular/common/http';
import { inject, Injectable } from '@angular/core';
import { ApiProblem } from 'shared';
import { firstValueFrom } from 'rxjs';
import type {
  HttpMethod,
  Operation,
  OptionsArgument,
  PathsWith,
  ResponseOf,
} from './admin-api-types';
import { SessionState } from './session-state';

type QueryValue = string | number | boolean;

/** A request before its types are checked away. */
interface RawOptions {
  readonly path?: Readonly<Record<string, string | number>>;
  readonly query?: Readonly<Record<string, QueryValue | undefined>>;
  readonly body?: unknown;
}

/** A file the admin server sent: its content and the name it gave it. */
export interface DownloadedFile {
  readonly blob: Blob;
  readonly fileName: string | null;
}

/** The header every change sends, with the session's token (support-admin.md, H11). */
export const CSRF_HEADER = 'X-CSRF-Token';
const UNAUTHENTICATED = 'admin.unauthenticated';

/** `/users/{id}` with `{ id: 'a b' }`: `/users/a%20b`. */
function expandPath(template: string, parameters: RawOptions['path'] = {}): string {
  return template.replace(/\{(\w+)\}/g, (_, name: string) =>
    encodeURIComponent(String(parameters[name])),
  );
}

function toParams(query: RawOptions['query'] = {}): HttpParams {
  const defined = Object.entries(query).filter(
    (entry): entry is [string, QueryValue] => entry[1] !== undefined && entry[1] !== '',
  );
  return new HttpParams({ fromObject: Object.fromEntries(defined) });
}

/** `attachment; filename="export.zip"`: `export.zip`. */
export function fileNameOf(disposition: string | null): string | null {
  const match = /filename="?([^";]+)"?/i.exec(disposition ?? '');
  return match ? match[1] : null;
}

/** A failed file download carries its problem as a `Blob`: read it back as JSON. */
async function readBlobProblem(error: unknown): Promise<unknown> {
  if (!(error instanceof HttpErrorResponse) || !(error.error instanceof Blob)) {
    return error;
  }
  try {
    const body: unknown = JSON.parse(await error.error.text());
    return new HttpErrorResponse({ error: body, status: error.status, url: error.url ?? '' });
  } catch {
    return error;
  }
}

/**
 * The admin API, typed by path and method from the generated `admin-api/schema.d.ts`: the
 * console's only user of `HttpClient`. Every change sends the session's CSRF token; a failure is
 * an `ApiProblem`, and `admin.unauthenticated` ends the session.
 */
@Injectable({ providedIn: 'root' })
export class AdminClient {
  private readonly http = inject(HttpClient);
  private readonly state = inject(SessionState);

  /** Sends the operation of `method` on `path`, and resolves to its JSON answer. */
  async request<M extends HttpMethod, P extends PathsWith<M>>(
    method: M,
    path: P,
    ...[options]: OptionsArgument<Operation<P, M>>
  ): Promise<ResponseOf<Operation<P, M>>> {
    const raw = (options ?? {}) as RawOptions;
    const request = this.http.request(method.toUpperCase(), expandPath(path, raw.path), {
      body: raw.body,
      headers: this.headersFor(method),
      params: toParams(raw.query),
      observe: 'body',
      responseType: 'json',
    });
    const answer = await this.settle(firstValueFrom(request));
    return (answer ?? undefined) as ResponseOf<Operation<P, M>>;
  }

  /** Downloads the file `GET path` answers, such as a screenshot. */
  async download<P extends PathsWith<'get'>>(
    path: P,
    parameters: Readonly<Record<string, string>>,
  ): Promise<DownloadedFile> {
    const request = this.http.get(expandPath(path, parameters), {
      observe: 'response',
      responseType: 'blob',
    });
    return this.toDownloadedFile(await this.settle(firstValueFrom(request)));
  }

  /** Downloads the file `POST path` answers, sending the session's CSRF token and `options`' body: a user's export. */
  async downloadPost<P extends PathsWith<'post'>>(
    path: P,
    ...[options]: OptionsArgument<Operation<P, 'post'>>
  ): Promise<DownloadedFile> {
    const raw = (options ?? {}) as RawOptions;
    const request = this.http.post(expandPath(path, raw.path), raw.body, {
      headers: this.headersFor('post'),
      observe: 'response',
      responseType: 'blob',
    });
    return this.toDownloadedFile(await this.settle(firstValueFrom(request)));
  }

  private toDownloadedFile(response: HttpResponse<Blob>): DownloadedFile {
    const blob = response.body ?? new Blob([]);
    return { blob, fileName: fileNameOf(response.headers.get('Content-Disposition')) };
  }

  private headersFor(method: HttpMethod): HttpHeaders {
    const token = this.state.csrfToken();
    return method === 'get' || token === null
      ? new HttpHeaders()
      : new HttpHeaders({ [CSRF_HEADER]: token });
  }

  private async settle<T>(answer: Promise<T>): Promise<T> {
    try {
      return await answer;
    } catch (error: unknown) {
      const problem = ApiProblem.from(await readBlobProblem(error));
      if (problem instanceof ApiProblem && problem.code === UNAUTHENTICATED) {
        this.state.end();
      }
      throw problem;
    }
  }
}
