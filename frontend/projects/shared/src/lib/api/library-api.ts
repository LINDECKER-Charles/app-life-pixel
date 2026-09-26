import { HttpClient, HttpErrorResponse, HttpResponse } from '@angular/common/http';
import { inject, Injectable } from '@angular/core';
import { firstValueFrom } from 'rxjs';
import { ApiClient } from './api-client';
import { API_HEADERS, LIFE_PIXEL_CLIENT, LIFE_PIXEL_CLIENT_HEADER } from './api-headers';
import { ApiProblem } from './api-problem';
import type { HttpMethod } from './api-types';
import type { components, operations } from './schema';

/** A project of the account's library (accounts.md, H6). */
export type Project = components['schemas']['Project'];
/** An animation of the account's library, without its document. */
export type Animation = components['schemas']['Animation'];
/** A change of an animation: its title, its project, or both. */
export type AnimationChange = components['schemas']['AnimationChange'];
/** A copy of an animation: its title, and its project when not the original's. */
export type AnimationCopy = components['schemas']['AnimationCopy'];
/** A page of projects, from the most recently updated. */
export type ProjectPage = components['schemas']['Page_Project'];
/** A page of animations, from the most recently updated. */
export type AnimationPage = components['schemas']['Page_Animation'];
/** The query of `GET /projects`: a cursor and a page size. */
export type ProjectQuery = NonNullable<operations['listProjects']['parameters']['query']>;
/** The query of `GET /animations`: a project, a search text, a cursor and a page size. */
export type AnimationQuery = NonNullable<operations['listAnimations']['parameters']['query']>;

/** A document with the version its `ETag` names. */
export interface VersionedDocument {
  readonly document: Uint8Array;
  readonly version: number;
}

/** The media type of an animation's document, as the local library's files hold it. */
export const ANIMATION_DOCUMENT_TYPE = 'application/vnd.life-pixel.animation+json';

const ANIMATIONS = '/api/v1/animations';

/** `"4"` or `W/"4"`: the version a document's `ETag` names. */
function versionOf(etag: string | null): number {
  const version = Number(etag?.replace(/^W\//, '').replaceAll('"', ''));
  if (!Number.isSafeInteger(version)) throw new ApiProblem(0, 'internal.error');
  return version;
}

/** An `If-Match` naming `version`. */
function ifMatch(version: number): Record<string, string> {
  return { 'If-Match': `"${version}"` };
}

/** The bytes of a document, in a buffer of their own: `HttpClient` sends a buffer as it is. */
function bufferOf(document: Uint8Array): ArrayBuffer {
  const copy = new Uint8Array(document.byteLength);
  copy.set(document);
  return copy.buffer;
}

/** Whether `value` is an `ArrayBuffer`, whatever the realm that made it (a worker, a test's). */
function isArrayBuffer(value: unknown): value is ArrayBuffer {
  return Object.prototype.toString.call(value) === '[object ArrayBuffer]';
}

/** The JSON of a problem's bytes, or `null` when they hold none. */
function parseBytes(bytes: ArrayBuffer): unknown {
  try {
    return JSON.parse(new TextDecoder().decode(bytes));
  } catch {
    return null;
  }
}

/** The `ApiProblem` of a failure, whose body came back as bytes when the request asked for them. */
function problemOf(error: unknown): unknown {
  if (error instanceof HttpErrorResponse && isArrayBuffer(error.error)) {
    const { status, statusText, headers } = error;
    const body = parseBytes(error.error);
    return ApiProblem.from(new HttpErrorResponse({ error: body, status, statusText, headers }));
  }
  return ApiProblem.from(error);
}

/** A request that the typed `ApiClient` cannot express: an `If-Match`, a document's bytes. */
interface DocumentRequest {
  readonly method: HttpMethod;
  readonly url: string;
  readonly body?: unknown;
  readonly headers?: Readonly<Record<string, string>>;
}

/**
 * H6's library routes (accounts.md, H6): projects, animations and their documents. The JSON routes
 * go through the typed `ApiClient`; the ones that carry a document's bytes, name a version in
 * `If-Match` or read one from `ETag` — which `ApiClient` does not express — go through
 * `HttpClient` here, with the same headers (`Life-Pixel-Client`, `API_HEADERS`) and the same
 * `ApiProblem` failures.
 */
@Injectable({ providedIn: 'root' })
export class LibraryApi {
  private readonly client = inject(ApiClient);
  private readonly http = inject(HttpClient);
  private readonly clientName = inject(LIFE_PIXEL_CLIENT);
  private readonly headerSources = inject(API_HEADERS, { optional: true }) ?? [];

  listProjects(query: ProjectQuery = {}): Promise<ProjectPage> {
    return this.client.request('get', '/api/v1/projects', { query });
  }

  createProject(name: string): Promise<Project> {
    return this.client.request('post', '/api/v1/projects', { body: { name } });
  }

  renameProject(id: string, name: string): Promise<Project> {
    return this.client.request('patch', '/api/v1/projects/{id}', { path: { id }, body: { name } });
  }

  duplicateProject(id: string, name: string): Promise<Project> {
    const path = { id };
    return this.client.request('post', '/api/v1/projects/{id}/duplicate', { path, body: { name } });
  }

  deleteProject(id: string): Promise<void> {
    return this.client.request('delete', '/api/v1/projects/{id}', { path: { id } });
  }

  listAnimations(query: AnimationQuery = {}): Promise<AnimationPage> {
    return this.client.request('get', '/api/v1/animations', { query });
  }

  getAnimation(id: string): Promise<Animation> {
    return this.client.request('get', '/api/v1/animations/{id}', { path: { id } });
  }

  /** Moves the animation: no version needed, since its document does not change. */
  moveAnimation(id: string, projectId: string): Promise<Animation> {
    const path = { id };
    return this.client.request('patch', '/api/v1/animations/{id}', { path, body: { projectId } });
  }

  duplicateAnimation(id: string, copy: AnimationCopy): Promise<Animation> {
    const path = { id };
    return this.client.request('post', '/api/v1/animations/{id}/duplicate', { path, body: copy });
  }

  deleteAnimation(id: string): Promise<void> {
    return this.client.request('delete', '/api/v1/animations/{id}', { path: { id } });
  }

  /** Retitles the animation: a save of its document, so under the version it replaces. */
  async renameAnimation(id: string, title: string, version: number): Promise<Animation> {
    const url = `${ANIMATIONS}/${encodeURIComponent(id)}`;
    const request = { method: 'patch', url, body: { title }, headers: ifMatch(version) } as const;
    return (await this.send(request, 'json')).body as Animation;
  }

  async createAnimation(projectId: string, document: Uint8Array): Promise<Animation> {
    const url = `/api/v1/projects/${encodeURIComponent(projectId)}/animations`;
    const request = { method: 'post', url, body: bufferOf(document) } as const;
    return (await this.send(request, 'json')).body as Animation;
  }

  async getDocument(id: string): Promise<VersionedDocument> {
    const url = `${ANIMATIONS}/${encodeURIComponent(id)}/document`;
    const response = await this.send({ method: 'get', url }, 'arraybuffer');
    const body = isArrayBuffer(response.body) ? response.body : new ArrayBuffer(0);
    return { document: new Uint8Array(body), version: versionOf(response.headers.get('ETag')) };
  }

  /** Replaces the document, when `version` is still the saved one (`document.version_conflict`). */
  async saveDocument(id: string, document: Uint8Array, version: number): Promise<Animation> {
    const url = `${ANIMATIONS}/${encodeURIComponent(id)}/document`;
    const body = bufferOf(document);
    const request = { method: 'put', url, body, headers: ifMatch(version) } as const;
    return (await this.send(request, 'json')).body as Animation;
  }

  private async send(
    request: DocumentRequest,
    responseType: 'json' | 'arraybuffer',
  ): Promise<HttpResponse<unknown>> {
    const answer = this.http.request(request.method.toUpperCase(), request.url, {
      body: request.body,
      headers: this.headersFor(request),
      observe: 'response',
      responseType: responseType as 'json',
    });
    try {
      return await firstValueFrom(answer);
    } catch (error: unknown) {
      throw problemOf(error);
    }
  }

  private headersFor(request: DocumentRequest): Record<string, string> {
    const info = { method: request.method, url: request.url };
    const base: Record<string, string> = { [LIFE_PIXEL_CLIENT_HEADER]: this.clientName };
    if (isArrayBuffer(request.body)) base['Content-Type'] = ANIMATION_DOCUMENT_TYPE;
    const merged = this.headerSources.reduce(
      (headers, source) => ({ ...headers, ...source(info) }),
      base,
    );
    return { ...merged, ...request.headers };
  }
}
