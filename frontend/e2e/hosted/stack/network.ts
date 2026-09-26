import type { ChildProcess } from 'node:child_process';
import { connect } from 'node:net';
import { setTimeout as sleep } from 'node:timers/promises';

/** How long a binary gets to answer `/healthz` once started: it migrates first. */
const HEALTHY_TIMEOUT_MS = 60_000;
const POLL_PERIOD_MS = 250;

/** Whether something accepts connections on `127.0.0.1:port`. */
function isListening(port: number): Promise<boolean> {
  return new Promise((done) => {
    const socket = connect({ host: '127.0.0.1', port });
    socket.once('connect', () => {
      socket.destroy();
      done(true);
    });
    socket.once('error', () => done(false));
  });
}

/** Fails, naming them, when any of `ports` is taken: the suite never shares a port. */
export async function expectFree(ports: readonly number[]): Promise<void> {
  const taken: number[] = [];
  for (const port of ports) if (await isListening(port)) taken.push(port);
  if (taken.length > 0) {
    throw new Error(`Ports ${taken.join(', ')} are in use: stop what listens there first.`);
  }
}

/** Waits until `child` answers 200 on `/healthz` of `port`; fails if it exits first. */
export async function waitHealthy(child: ChildProcess, port: number, name: string): Promise<void> {
  const deadline = Date.now() + HEALTHY_TIMEOUT_MS;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) {
      throw new Error(`${name} exited with ${child.exitCode}: see e2e/hosted/logs/.`);
    }
    const healthy = await fetch(`http://127.0.0.1:${port}/healthz`)
      .then((response) => response.ok)
      .catch(() => false);
    if (healthy) return;
    await sleep(POLL_PERIOD_MS);
  }
  throw new Error(`${name} did not answer /healthz within ${HEALTHY_TIMEOUT_MS} ms.`);
}
