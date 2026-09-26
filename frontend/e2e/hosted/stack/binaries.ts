import { execFile, spawn, type ChildProcess } from 'node:child_process';
import { createWriteStream, existsSync } from 'node:fs';
import { mkdir } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { once } from 'node:events';

/** The repository's root: the binaries run there, where they read `.env`. */
export const REPOSITORY_ROOT = resolve(__dirname, '../../../..');
/** Where the binaries' output lands, one file each, for the traces of a failed run. */
export const LOGS_DIR = resolve(__dirname, '../logs');

/** The two binaries the suite runs. */
export type Binary = 'life-pixel-server' | 'life-pixel-admin-server';

/** How long a binary gets to stop on SIGTERM before it is killed. */
const STOP_TIMEOUT_MS = 10_000;
/** Enough for any output of a subcommand the suite reads. */
const MAX_OUTPUT_BYTES = 1024 * 1024;

const FEATURES = 'life-pixel-server/stack-tests,life-pixel-admin-server/stack-tests';

/** The debug build of `binary`, compiled with `stack-tests` for its `test-database` subcommand. */
function binaryPath(binary: Binary): string {
  const target = process.env['CARGO_TARGET_DIR'] ?? join(REPOSITORY_ROOT, 'target');
  return resolve(REPOSITORY_ROOT, target, 'debug', binary);
}

/** Builds both binaries with `stack-tests`: nothing to do when they are current. */
export async function buildBinaries(): Promise<void> {
  const args = ['build', '--locked', '-p', 'life-pixel-server', '-p', 'life-pixel-admin-server'];
  const cargo = spawn('cargo', [...args, '--features', FEATURES], {
    cwd: REPOSITORY_ROOT,
    stdio: ['ignore', 'inherit', 'inherit'],
  });
  const [code] = (await once(cargo, 'exit')) as [number | null];
  if (code !== 0) throw new Error(`cargo build exited with ${code}`);
  for (const binary of ['life-pixel-server', 'life-pixel-admin-server'] as const) {
    if (!existsSync(binaryPath(binary))) throw new Error(`${binaryPath(binary)} is missing`);
  }
}

/** Runs a subcommand of `binary` to its end, and returns its standard output. */
export function runBinary(
  binary: Binary,
  args: readonly string[],
  options: { env?: NodeJS.ProcessEnv; input?: string } = {},
): Promise<string> {
  return new Promise((done, fail) => {
    const child = execFile(
      binaryPath(binary),
      args,
      { cwd: REPOSITORY_ROOT, env: options.env ?? process.env, maxBuffer: MAX_OUTPUT_BYTES },
      (error, stdout, stderr) => {
        if (error) fail(new Error(`${binary} ${args.join(' ')} failed: ${stderr || error}`));
        else done(stdout);
      },
    );
    child.stdin?.end(options.input ?? '');
  });
}

/** Starts `binary serve` with `env`, its output written to `logs/<binary>.log`. */
export async function startBinary(binary: Binary, env: NodeJS.ProcessEnv): Promise<ChildProcess> {
  await mkdir(LOGS_DIR, { recursive: true });
  const log = createWriteStream(join(LOGS_DIR, `${binary}.log`));
  const child = spawn(binaryPath(binary), ['serve'], {
    cwd: REPOSITORY_ROOT,
    env,
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  child.stdout?.pipe(log);
  child.stderr?.pipe(log);
  return child;
}

/** Stops a binary started by `startBinary`: SIGTERM, then SIGKILL if it lingers. */
export async function stopBinary(child: ChildProcess): Promise<void> {
  if (child.exitCode !== null || child.signalCode !== null) return;
  const exited = once(child, 'exit');
  child.kill('SIGTERM');
  const timer = setTimeout(() => child.kill('SIGKILL'), STOP_TIMEOUT_MS);
  await exited;
  clearTimeout(timer);
}
