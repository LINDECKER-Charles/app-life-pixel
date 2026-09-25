// Builds the editor's engine before the app is served, built or tested against it:
// `cargo xtask build-editor` writes the JavaScript glue to src/app/engine/wasm/generated/ and the
// module to public/engine/ (editor.md, W1). With LP_ENGINE_PREBUILT=1 and both outputs present —
// CI and the Docker build prepare them once — it builds nothing.
import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const REPOSITORY = fileURLToPath(new URL('../../', import.meta.url));
const OUTPUTS = [
  'projects/app/src/app/engine/wasm/generated/editor_engine.js',
  'projects/app/src/app/engine/wasm/generated/editor_engine.d.ts',
  'projects/app/public/engine/editor_engine_bg.wasm',
].map((output) => new URL(`../${output}`, import.meta.url));

const isPrebuilt = process.env.LP_ENGINE_PREBUILT === '1';

if (isPrebuilt && OUTPUTS.every((output) => existsSync(output))) {
  console.log('prebuild: LP_ENGINE_PREBUILT=1 and the engine is built: nothing to do.');
} else {
  if (isPrebuilt) console.warn('prebuild: LP_ENGINE_PREBUILT=1 but the engine is missing.');
  const { status, error } = spawnSync('cargo', ['xtask', 'build-editor'], {
    cwd: REPOSITORY,
    stdio: 'inherit',
  });
  if (error) {
    console.error(
      `prebuild: cannot run cargo (${error.message}): install Rust with rustup, or set ` +
        'LP_ENGINE_PREBUILT=1 with the outputs of `cargo xtask build-editor` in place.',
    );
  }
  process.exitCode = status ?? 1;
}
