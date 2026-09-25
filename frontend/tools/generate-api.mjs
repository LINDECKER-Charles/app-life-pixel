// Regenerates the API descriptions and their types (docs/v1/server.md, H3): each server prints
// its OpenAPI description, committed beside its crate, then openapi-typescript turns it into the
// schema its typed client reads. CI's `api` job runs it and fails on any difference.
import { execFileSync } from 'node:child_process';
import { writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const REPO_DIR = fileURLToPath(new URL('../../', import.meta.url));
const FRONTEND_DIR = fileURLToPath(new URL('../', import.meta.url));
const OPENAPI_TYPESCRIPT = fileURLToPath(
  new URL('../node_modules/openapi-typescript/bin/cli.js', import.meta.url),
);
const PRETTIER = fileURLToPath(
  new URL('../node_modules/prettier/bin/prettier.cjs', import.meta.url),
);

// One entry per description, H11 adds the admin server's: the binary that prints it, the
// committed JSON, and its types, from the root of the repository.
const DESCRIPTIONS = [
  {
    binary: 'life-pixel-server',
    json: 'crates/server/openapi.json',
    types: 'frontend/projects/shared/src/lib/api/schema.d.ts',
  },
];

/** The description `binary openapi` prints, built from the locked dependencies. */
function printDescription(binary) {
  const args = ['run', '--quiet', '--locked', '--package', binary, '--', 'openapi'];
  return execFileSync('cargo', args, {
    cwd: REPO_DIR,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'inherit'],
  });
}

/** Writes the types of the description at `json` to `types`, formatted as the frontend is. */
function generateTypes(json, types) {
  const output = `${REPO_DIR}${types}`;
  execFileSync(process.execPath, [OPENAPI_TYPESCRIPT, `${REPO_DIR}${json}`, '--output', output], {
    cwd: FRONTEND_DIR,
    stdio: 'inherit',
  });
  execFileSync(process.execPath, [PRETTIER, '--write', '--log-level', 'warn', output], {
    cwd: FRONTEND_DIR,
    stdio: 'inherit',
  });
}

for (const { binary, json, types } of DESCRIPTIONS) {
  writeFileSync(`${REPO_DIR}${json}`, printDescription(binary));
  generateTypes(json, types);
  console.log(`${json} and ${types} regenerated.`);
}
