import type { ProblemParams } from 'shared';
import type { FormError } from './form-errors';

/** A form's own check: the message `key` when `valid` is false, nothing otherwise. */
export function fieldError(
  valid: boolean,
  key: string,
  params: ProblemParams = {},
): FormError | undefined {
  return valid ? undefined : { key, params };
}
