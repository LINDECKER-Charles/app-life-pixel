import type { Limits, LoopMode } from '../../engine/engine-types';
import type { TagDialogTarget } from './tag-dialog-state';

export interface TagFormValues {
  readonly name: string;
  readonly first: number;
  readonly last: number;
  readonly loop: LoopMode;
}

/** The form's starting values: an existing tag's, or a blank name over the target range. */
export function initialValues(target: TagDialogTarget): TagFormValues {
  if (target.kind === 'edit') return { ...target.tag };
  return { name: '', first: target.first, last: target.last, loop: 'loop' };
}

/** A tag name holds 1 to `tagNameMaxChars` characters once trimmed (core.md's `Tag`). */
export function isValidTagName(name: string, limits: Limits): boolean {
  const length = [...name.trim()].length;
  return length > 0 && length <= limits.tagNameMaxChars;
}

/** A tag's range: both ends within the frames, first no later than last. */
export function isValidRange(first: number, last: number, frameCount: number): boolean {
  return (
    Number.isInteger(first) &&
    Number.isInteger(last) &&
    first >= 0 &&
    last >= first &&
    last < frameCount
  );
}
