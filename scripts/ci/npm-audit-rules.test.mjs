import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, it } from 'node:test';
import {
  MAX_ALLOWANCE_DAYS,
  advisoriesOf,
  allowListProblemsOf,
  auditOutcomeOf,
} from './npm-audit-rules.mjs';

const SCRIPT = fileURLToPath(new URL('./check-npm-audit.mjs', import.meta.url));
const TODAY = '2026-10-15';
const GHSA = 'GHSA-vfj7-8cjw-p6xm';
const OTHER = 'GHSA-68fv-2mgg-jv7q';
const advisory = (name, id, severity) => ({
  source: 1241209,
  name,
  title: `${name} advisory`,
  url: `https://github.com/advisories/${id}`,
  severity,
});
// The shape of `npm audit --json` (npm 7+): the advisory under its own package, the packages
// that depend on it naming that package only.
const REPORT = {
  vulnerabilities: {
    braces: { name: 'braces', via: [advisory('braces', GHSA, 'high')] },
    micromatch: { name: 'micromatch', via: ['braces'] },
    'source-map-js': { name: 'source-map-js', via: [advisory('source-map-js', OTHER, 'low')] },
  },
};
const entry = (fields = {}) => ({
  advisory: GHSA,
  package: 'braces',
  reason: 'reached only by stylelint, on the globs of the repository',
  expires: '2026-12-31',
  ...fields,
});

describe('advisoriesOf', () => {
  it('reads each advisory once, with its GHSA id, and skips the dependents', () => {
    assert.deepEqual(
      advisoriesOf(REPORT).map(({ id, package: name, severity }) => [id, name, severity]),
      [
        [GHSA, 'braces', 'high'],
        [OTHER, 'source-map-js', 'low'],
      ],
    );
  });

  it('falls back on the npm source id when the advisory has no GHSA URL', () => {
    const withoutUrl = { ...advisory('a', 'x', 'high'), url: '' };
    const report = { vulnerabilities: { a: { via: [withoutUrl] } } };

    assert.equal(advisoriesOf(report)[0].id, '1241209');
  });
});

describe('allowListProblemsOf', () => {
  it('accepts complete entries that expire within the allowance', () => {
    assert.deepEqual(allowListProblemsOf([entry()], TODAY), []);
  });

  it('refuses what is not an array', () => {
    assert.equal(allowListProblemsOf({}, TODAY)[0].problem, 'the allow-list is not a JSON array');
  });

  it('names the missing fields', () => {
    const [problem] = allowListProblemsOf([entry({ reason: '', expires: undefined })], TODAY);

    assert.deepEqual(problem, { index: 0, problem: 'missing reason, expires' });
  });

  it('refuses a date that is not a calendar date', () => {
    for (const expires of ['31/12/2026', '2026-02-30', 'soon']) {
      const [problem] = allowListProblemsOf([entry({ expires })], TODAY);

      assert.match(problem.problem, /is not a YYYY-MM-DD date/, expires);
    }
  });

  it('refuses an expiry beyond the allowance', () => {
    const [problem] = allowListProblemsOf([entry({ expires: '2027-01-14' })], TODAY);

    assert.match(problem.problem, new RegExp(`more than ${MAX_ALLOWANCE_DAYS}`));
    assert.deepEqual(allowListProblemsOf([entry({ expires: '2027-01-13' })], TODAY), []);
  });
});

const outcome = (entries, level = 'moderate') =>
  auditOutcomeOf(REPORT, entries, { level, today: TODAY });

describe('auditOutcomeOf', () => {
  it('blocks an advisory at or above the level that no entry names', () => {
    const { blocking, allowed } = outcome([]);

    assert.deepEqual(
      blocking.map((item) => [item.id, item.expired]),
      [[GHSA, null]],
    );
    assert.deepEqual(allowed, []);
  });

  it('ignores the advisories below the level', () => {
    assert.deepEqual(outcome([], 'critical').blocking, []);
    assert.equal(outcome([], 'low').blocking.length, 2);
  });
});

describe('auditOutcomeOf, with entries', () => {
  it('accepts an advisory its entry names, up to and including the expiry day', () => {
    const { blocking, allowed } = outcome([entry({ expires: TODAY })]);

    assert.deepEqual(blocking, []);
    assert.deepEqual(
      allowed.map((item) => [item.id, item.expires]),
      [[GHSA, TODAY]],
    );
  });

  it('blocks again once the entry expired, and says so', () => {
    const { blocking } = outcome([entry({ expires: '2026-10-14' })]);

    assert.deepEqual(
      blocking.map((item) => item.expired),
      ['2026-10-14'],
    );
  });

  it('does not accept the advisory for another package', () => {
    assert.equal(outcome([entry({ package: 'micromatch' })]).blocking.length, 1);
  });

  it('lists the entries that name nothing reported at the level', () => {
    const gone = entry({ advisory: 'GHSA-2222-3333-4444' });

    assert.deepEqual(outcome([entry(), gone]).stale, [gone]);
  });
});

const serialised = (value) => (typeof value === 'string' ? value : JSON.stringify(value));

// Runs the script on a temporary directory that holds the allow-list given, if any.
function check({ allowList, report = REPORT, args = [] } = {}) {
  const directory = mkdtempSync(join(tmpdir(), 'life-pixel-npm-audit-'));
  try {
    if (allowList) {
      writeFileSync(join(directory, 'npm-audit-allowlist.json'), serialised(allowList));
    }
    const reportPath = join(directory, 'report.json');
    writeFileSync(reportPath, serialised(report));
    const options = [directory, '--report', reportPath, '--today', TODAY, ...args];
    return spawnSync(process.execPath, [SCRIPT, ...options], { encoding: 'utf8' });
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

describe('check-npm-audit, without an allow-list', () => {
  it('fails on an advisory at or above moderate, and names it', () => {
    const result = check();

    assert.equal(result.status, 1);
    assert.match(result.stderr, new RegExp(`braces: ${GHSA} \\(high\\).*not allowed`));
    assert.match(result.stderr, /1 advisories at or above moderate block/);
  });

  it('follows the audit level', () => {
    assert.equal(check({ args: ['--audit-level', 'critical'] }).status, 0);
    assert.match(check({ args: ['--audit-level', 'severe'] }).stderr, /must be one of/);
  });

  it('fails when npm gave no report, or reported an error', () => {
    assert.match(check({ report: 'npm ERR! network' }).stderr, /gave no JSON report/);
    const failed = check({ report: { error: { summary: 'audit endpoint returned 503' } } });

    assert.equal(failed.status, 1);
    assert.match(failed.stderr, /npm audit failed: audit endpoint returned 503/);
  });
});

describe('check-npm-audit, with an allow-list', () => {
  it('passes when an entry accepts the advisory, and prints until when', () => {
    const result = check({ allowList: [entry()] });

    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, new RegExp(`allowed until 2026-12-31: braces: ${GHSA}`));
  });

  it('fails once the entry expired', () => {
    const result = check({ allowList: [entry({ expires: '2026-10-01' })] });

    assert.equal(result.status, 1);
    assert.match(result.stderr, /allowed until 2026-10-01, expired/);
  });

  it('fails on a malformed allow-list, which then accepts nothing', () => {
    const result = check({ allowList: [entry({ reason: '' })] });

    assert.equal(result.status, 1);
    assert.match(result.stderr, /entry 0: missing reason/);
    assert.match(result.stderr, /not allowed/);
  });

  it('warns about an entry that names nothing reported', () => {
    const result = check({ allowList: [entry()], report: { vulnerabilities: {} } });

    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, new RegExp(`::warning::.*${GHSA} .* remove the entry`));
  });
});
