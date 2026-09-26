/**
 * The library's vocabulary, whatever stores it: the hosted account's (`HttpLibraryStore`) or the
 * desktop's local folder (T2). Shapes match H6's `Project`, `Animation` and `Page`.
 */

/** A project of the library, with how many animations it holds. */
export interface Project {
  readonly id: string;
  readonly name: string;
  readonly animationCount: number;
  readonly createdAt: string;
  readonly updatedAt: string;
}

/** An animation of the library, without its document; `version` is the one a save names. */
export interface AnimationSummary {
  readonly id: string;
  readonly projectId: string;
  readonly title: string;
  readonly width: number;
  readonly height: number;
  readonly frameCount: number;
  readonly documentBytes: number;
  readonly version: number;
  readonly createdAt: string;
  readonly updatedAt: string;
}

/** A page of a list, from the most recently updated item; `nextCursor` is `null` on the last. */
export interface Page<T> {
  readonly items: readonly T[];
  readonly nextCursor: string | null;
}

/** Which page to read: after `cursor`, at most `limit` items. */
export interface PageQuery {
  readonly cursor?: string;
  readonly limit?: number;
}

/** Which animations to list: those of a project, those whose title holds `query`, or all. */
export interface AnimationFilter {
  readonly projectId?: string;
  readonly query?: string;
}

/** How a library operation fails: a code the interface translates as `errors.<code>`. */
export interface LibraryFailure {
  readonly code: string;
  readonly params: Readonly<Record<string, unknown>>;
}

/** The code of any failure a store could not name. */
export const UNKNOWN_FAILURE_CODE = 'internal.error';

/** Whether `error` is a `LibraryFailure`, as every `LibraryStore` rejects with. */
export function isLibraryFailure(error: unknown): error is LibraryFailure {
  if (typeof error !== 'object' || error === null) return false;
  const { code, params } = error as Partial<Record<keyof LibraryFailure, unknown>>;
  return typeof code === 'string' && typeof params === 'object' && params !== null;
}

/** `error` as a `LibraryFailure`: itself when it is one, `internal.error` otherwise. */
export function toLibraryFailure(error: unknown): LibraryFailure {
  if (isLibraryFailure(error)) return { code: error.code, params: error.params };
  return { code: UNKNOWN_FAILURE_CODE, params: {} };
}
