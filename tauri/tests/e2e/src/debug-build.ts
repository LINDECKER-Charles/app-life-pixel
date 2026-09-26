import { existsSync } from 'node:fs';
import path from 'node:path';

/** The repository's root: this package lives in tauri/tests/e2e/. */
const REPOSITORY_ROOT = path.resolve(import.meta.dirname, '../../../..');
/** The desktop app's binary, as life-pixel-desktop's Cargo.toml names it. */
const APPLICATION_BINARY = 'life-pixel-desktop';
/** The bundled CLI, which tauri-build copies beside the app from bundle.conf.json's externalBin. */
const CLI_BINARY = 'life-pixel';

/** The binaries of the debug app that `npm run build` makes. */
export interface DebugBuild {
  /** The app, which tauri-driver starts. */
  readonly application: string;
  /** The `life-pixel` sidecar beside it. */
  readonly cli: string;
}

/** The debug build in Cargo's target folder; fails with what to run when it is missing. */
export function debugBuild(): DebugBuild {
  const target = process.env['CARGO_TARGET_DIR'] ?? path.join(REPOSITORY_ROOT, 'target');
  const folder = path.join(target, 'debug');
  const build = {
    application: path.join(folder, APPLICATION_BINARY),
    cli: path.join(folder, CLI_BINARY),
  };
  for (const binary of Object.values(build)) {
    if (!existsSync(binary)) {
      throw new Error(`${binary} is missing: run \`npm run build\` in tauri/tests/e2e first.`);
    }
  }
  return build;
}
