import { clearMocks } from '@tauri-apps/api/mocks';

/**
 * Undoes `mockIPC()` for the next test: `clearMocks()` only empties `window.__TAURI_INTERNALS__`,
 * it never removes it, so `isDesktop()` would stay `true` for the rest of the file. Call this in
 * every `afterEach` of a test that calls `mockIPC()`.
 */
export function clearTauriMocks(): void {
  clearMocks();
  delete (window as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
}
