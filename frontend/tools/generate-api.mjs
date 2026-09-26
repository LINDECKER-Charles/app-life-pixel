// Regenerates the API descriptions and their types (docs/v1/server.md, H3): each server prints
// its OpenAPI description, committed beside its crate, then openapi-typescript turns it into the
// schema its typed client reads. The internal admin API's description (H10) has no types of its
// own: the admin server (H11) relays it, and its description — its own routes and the relayed
// ones, under /api/admin/v1 — has the console's types. It reads the server's at build time, so
// it comes after it. CI's `api` job runs it and fails on any difference.
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

// One entry per description: the binary and the command that print it, the committed JSON, and
// its types if any, from the root of the repository.
const DESCRIPTIONS = [
  {
    binary: 'life-pixel-server',
    command: 'openapi',
    json: 'crates/server/openapi.json',
    types: 'frontend/projects/shared/src/lib/api/schema.d.ts',
  },
  {
    binary: 'life-pixel-server',
    command: 'admin-openapi',
    json: 'crates/server/admin-openapi.json',
  },
  {
    binary: 'life-pixel-admin-server',
    command: 'openapi',
    json: 'crates/admin-server/openapi.json',
    types: 'frontend/projects/shared/src/lib/admin-api/schema.d.ts',
  },
];

/** The description `binary command` prints, built from the locked dependencies. */
function printDescription(binary, command) {
  const args = ['run', '--quiet', '--locked', '--package', binary, '--', command];
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

for (const { binary, command, json, types } of DESCRIPTIONS) {
  writeFileSync(`${REPO_DIR}${json}`, printDescription(binary, command));
  if (types === undefined) {
    console.log(`${json} regenerated.`);
    continue;
  }
  generateTypes(json, types);
  console.log(`${json} and ${types} regenerated.`);
}
