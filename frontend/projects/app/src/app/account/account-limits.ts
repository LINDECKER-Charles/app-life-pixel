/**
 * `core::limits`, transcribed (accounts.md, H7 "the lengths from the engine's limits"): only the
 * account forms' fields need them, so this avoids loading the editor's wasm engine — `EngineStore`
 * starts it — on pages that never open an animation. Mirrors `engine/testing/mock-limits.ts`,
 * written for the same reason.
 */
export const ACCOUNT_LIMITS = {
  passwordMinChars: 12,
  passwordMaxChars: 128,
  emailMaxChars: 254,
} as const;
