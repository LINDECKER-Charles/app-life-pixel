import type { Limits } from '../../engine/engine-types';

/** `errors.import.image_too_large`'s parameters, shown without ever reading the oversized file. */
export interface ImportTooLarge {
  readonly maxSide: number;
  readonly maxBytes: number;
}

/** Whether a picked file is too large to read at all, checked from `File.size` alone. */
export function isFileTooLarge(file: File, limits: Limits): ImportTooLarge | null {
  return file.size > limits.importMaxBytes
    ? { maxSide: limits.importMaxSide, maxBytes: limits.importMaxBytes }
    : null;
}
