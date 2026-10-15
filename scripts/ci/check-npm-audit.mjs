// Fails when a dependency of an npm package — development ones included — has an advisory at or
// above the audit level that the package's allow-list does not accept (npm-audit-rules.mjs):
//
//   node scripts/ci/check-npm-audit.mjs DIRECTORY [--audit-level LEVEL] [--report FILE]
//                                       [--today YYYY-MM-DD]
//
// Runs `npm audit --package-lock-only --json` in DIRECTORY: the lockfile is enough, no install is
// needed, and nothing runs `npm audit fix`. The allow-list is DIRECTORY/npm-audit-allowlist.json;
// without one, nothing is accepted. --report reads a saved report instead of running npm;
// --today fixes the date the expiries are compared with. Run by the `npm-audit` job of
// `security.yml`, once per audited directory.

import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { parseArgs } from 'node:util';
import { SEVERITIES, allowListProblemsOf, auditOutcomeOf } from './npm-audit-rules.mjs';

const ALLOW_LIST_FILE = 'npm-audit-allowlist.json';
const DEFAULT_AUDIT_LEVEL = 'moderate';
const AUDIT_ARGS = ['audit', '--package-lock-only', '--json'];
const IS_WINDOWS = process.platform === 'win32';
const MAX_REPORT_BYTES = 64 * 1024 * 1024;

function auditReportOf(directory) {
  // npm exits non-zero whenever it finds an advisory: the report decides, not the status.
  const result = spawnSync(IS_WINDOWS ? 'npm.cmd' : 'npm', AUDIT_ARGS, {
    cwd: directory,
    encoding: 'utf8',
    maxBuffer: MAX_REPORT_BYTES,
    shell: IS_WINDOWS,
  });
  if (result.error) {
    throw result.error;
  }
  return parseReport(result.stdout, result.stderr);
}

function parseReport(text, detail = '') {
  let report;
  try {
    report = JSON.parse(text);
  } catch {
    throw new Error(`npm audit gave no JSON report ${detail.trim()}`.trim());
  }
  if (report.error) {
    throw new Error(`npm audit failed: ${report.error.summary ?? JSON.stringify(report.error)}`);
  }
  return report;
}

// An absent allow-list accepts nothing; an unreadable one fails the check.
function readAllowList(path) {
  return existsSync(path) ? JSON.parse(readFileSync(path, 'utf8')) : [];
}

const describe = (item) => `${item.package}: ${item.id} (${item.severity}) ${item.title}`;

function print(outcome, problems, allowList) {
  for (const { index, problem } of problems) {
    console.error(`${allowList}: entry ${index}: ${problem}`);
  }
  for (const item of outcome.blocking) {
    const why = item.expired ? `allowed until ${item.expired}, expired` : 'not allowed';
    console.error(`${describe(item)} — ${why} — ${item.url}`);
  }
  for (const item of outcome.allowed) {
    console.log(`allowed until ${item.expires}: ${describe(item)}`);
  }
  for (const entry of outcome.stale) {
    console.log(
      `::warning::${allowList}: ${entry.advisory} (${entry.package}) is no longer` +
        ' reported at this level: remove the entry.',
    );
  }
}

function optionsOf() {
  const { values, positionals } = parseArgs({
    allowPositionals: true,
    options: {
      'audit-level': { type: 'string', default: DEFAULT_AUDIT_LEVEL },
      report: { type: 'string' },
      today: { type: 'string', default: new Date().toISOString().slice(0, 10) },
    },
  });
  if (positionals.length !== 1) {
    throw new Error('usage: check-npm-audit.mjs DIRECTORY [--audit-level LEVEL]');
  }
  if (!SEVERITIES.includes(values['audit-level'])) {
    throw new Error(`--audit-level must be one of ${SEVERITIES.join(', ')}`);
  }
  return { ...values, directory: positionals[0] };
}

function main() {
  const options = optionsOf();
  const level = options['audit-level'];
  const allowList = join(options.directory, ALLOW_LIST_FILE);
  const entries = readAllowList(allowList);
  const problems = allowListProblemsOf(entries, options.today);
  const report = options.report
    ? parseReport(readFileSync(options.report, 'utf8'))
    : auditReportOf(options.directory);
  // A malformed allow-list accepts nothing.
  const accepted = problems.length === 0 ? entries : [];
  const outcome = auditOutcomeOf(report, accepted, { level, today: options.today });
  print(outcome, problems, allowList);
  if (problems.length > 0 || outcome.blocking.length > 0) {
    throw new Error(
      `${outcome.blocking.length} advisories at or above ${level} block, ` +
        `${problems.length} malformed allow-list entries`,
    );
  }
  console.log(
    `check-npm-audit: no unaccepted advisory at or above ${level} in ${options.directory}` +
      ` (${outcome.allowed.length} allowed)`,
  );
}

try {
  main();
} catch (error) {
  console.error(`check-npm-audit: ${error.message}`);
  process.exitCode = 1;
}
