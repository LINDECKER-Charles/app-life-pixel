import { HttpErrorResponse } from '@angular/common/http';

/** The code of a failure the server did not answer: the network, or a proxy in front of it. */
export const SERVICE_UNAVAILABLE = 'service.unavailable';
/** The code of an answer that is neither a success nor a problem. */
export const INTERNAL_ERROR = 'internal.error';

/** The statuses a proxy answers with when the server cannot: the network's is `0`. */
const UNAVAILABLE_STATUSES: readonly number[] = [0, 502, 503, 504];

/** The parameters of a problem's message, with camelCase keys. */
export type ProblemParams = Readonly<Record<string, unknown>>;

interface ProblemBody {
  readonly code: string;
  readonly params: ProblemParams;
}

function isProblemBody(body: unknown): body is ProblemBody {
  if (typeof body !== 'object' || body === null) {
    return false;
  }
  const { code, params } = body as Partial<Record<keyof ProblemBody, unknown>>;
  return typeof code === 'string' && typeof params === 'object' && params !== null;
}

/**
 * A failed API request: the status, and the code and params the interface translates as
 * `errors.<code>`, never a sentence from the server.
 */
export class ApiProblem extends Error {
  constructor(
    readonly status: number,
    readonly code: string,
    readonly params: ProblemParams = {},
  ) {
    super(code);
    this.name = 'ApiProblem';
  }

  /** The problem of a failed `HttpClient` request; any other error is returned as it is. */
  static from(error: unknown): unknown {
    if (!(error instanceof HttpErrorResponse)) {
      return error;
    }
    if (isProblemBody(error.error)) {
      return new ApiProblem(error.status, error.error.code, error.error.params);
    }
    const unavailable = UNAVAILABLE_STATUSES.includes(error.status);
    return new ApiProblem(error.status, unavailable ? SERVICE_UNAVAILABLE : INTERNAL_ERROR);
  }
}
