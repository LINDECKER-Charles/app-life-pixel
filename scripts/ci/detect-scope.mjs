// Writes what a change calls for as step outputs: `verify`, false when a pull request changes
// documentation alone. Only a pull request is narrowed: a push checks everything. Run by the
// `scope` jobs, which pass EVENT_NAME, and BASE_SHA and HEAD_SHA for a pull request; writes to
// GITHUB_OUTPUT.

import { execFileSync } from 'node:child_process';
import { appendFileSync } from 'node:fs';
import { needsVerification } from './verification-scope.mjs';

const PULL_REQUEST_VARIABLES = ['BASE_SHA', 'HEAD_SHA'];

function readEnvironment() {
  const required = [
    'EVENT_NAME',
    'GITHUB_OUTPUT',
    ...(process.env.EVENT_NAME === 'pull_request' ? PULL_REQUEST_VARIABLES : []),
  ];
  const missing = required.filter((name) => !process.env[name]);
  if (missing.length > 0) {
    throw new Error(`missing environment variable(s): ${missing.join(', ')}`);
  }
  return process.env;
}

// --no-renames: a file moved out of the code into docs/ still counts on the side it left.
function changedPaths(baseSha, headSha) {
  const names = execFileSync(
    'git',
    ['diff', '--name-only', '--no-renames', `${baseSha}...${headSha}`],
    { encoding: 'utf8' },
  );
  return names.split('\n').filter(Boolean);
}

function scopeOf(environment) {
  if (environment.EVENT_NAME !== 'pull_request') {
    return { verify: true };
  }
  const paths = changedPaths(environment.BASE_SHA, environment.HEAD_SHA);
  return { verify: paths.some(needsVerification) };
}

function main() {
  const environment = readEnvironment();
  const { verify } = scopeOf(environment);
  const outputs = `verify=${verify}\n`;
  process.stdout.write(outputs);
  appendFileSync(environment.GITHUB_OUTPUT, outputs);
}

main();
