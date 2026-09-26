import { ApiProblem, INTERNAL_ERROR, SERVICE_UNAVAILABLE } from 'shared';

/** A message to translate: its key and its parameters. */
export interface Message {
  readonly key: string;
  readonly params: Readonly<Record<string, unknown>>;
}

/** Why a panel or a table shows nothing: never a blank that looks like a dead stack. */
export type EmptyReason = 'nothing' | 'not_configured' | 'unavailable' | 'failed';

const NOT_CONFIGURED = 'monitoring.not_configured';
const UNAVAILABLE = ['monitoring.unavailable', SERVICE_UNAVAILABLE];

/** The problem of a failed call: an `ApiProblem` as it is, anything else as an internal error. */
export function asProblem(error: unknown): ApiProblem {
  return error instanceof ApiProblem ? error : new ApiProblem(0, INTERNAL_ERROR);
}

/** The message of a problem: `errors.<code>`, with its parameters. */
export function problemMessage(problem: ApiProblem): Message {
  return { key: `errors.${problem.code}`, params: problem.params };
}

/** Why a view is empty: nothing to show, or a source not configured, unreachable or failing. */
export function emptyReason(problem: ApiProblem | null): EmptyReason {
  if (problem === null) {
    return 'nothing';
  }
  if (problem.code === NOT_CONFIGURED) {
    return 'not_configured';
  }
  return UNAVAILABLE.includes(problem.code) ? 'unavailable' : 'failed';
}
