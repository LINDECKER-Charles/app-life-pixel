import { type ChildProcess, spawn } from 'node:child_process';
import { once } from 'node:events';
import { mkdtemp, rm } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';
import { Builder, Capabilities, type WebDriver } from 'selenium-webdriver';

/** tauri-driver's port, and the one of the native WebDriver behind it (the suites' 8460–8464). */
const DRIVER_PORT = '8460';
const NATIVE_DRIVER_PORT = '8461';
const DRIVER_URL = `http://127.0.0.1:${DRIVER_PORT}/`;
/** How long tauri-driver may take to answer, and how often to ask. */
const DRIVER_START_TIMEOUT_MS = 30_000;
const DRIVER_POLL_INTERVAL_MS = 250;
/** The browser name tauri-driver expects for a Tauri app. */
const TAURI_BROWSER_NAME = 'wry';
/** The interface's language, whatever the machine's. */
const LOCALE = 'C.UTF-8';
/** The app may still write its caches while it closes: removing them is tried again. */
const REMOVAL_RETRIES = 5;

/** The temporary folders of one run, so that it leaves the person's library and settings alone. */
export interface SessionFolders {
  /** The parent of the others, removed at the end. */
  readonly root: string;
  /** `LIFE_PIXEL_LIBRARY`: the library the app opens. */
  readonly library: string;
  /** `LP_EXPORT_DIR`: where the debug app saves exports, without a dialog. */
  readonly exports: string;
  /** The XDG folders' parent: the app's settings and the webview's storage. */
  readonly home: string;
}

/** The debug app started through tauri-driver, on temporary folders. */
export class DesktopSession {
  readonly driver: WebDriver;
  readonly folders: SessionFolders;
  readonly #tauriDriver: ChildProcess;

  private constructor(driver: WebDriver, folders: SessionFolders, tauriDriver: ChildProcess) {
    this.driver = driver;
    this.folders = folders;
    this.#tauriDriver = tauriDriver;
  }

  /** Starts tauri-driver, then the app at `application` in a new WebDriver session. */
  static async start(application: string): Promise<DesktopSession> {
    const folders = await temporaryFolders();
    const tauriDriver = startTauriDriver(folders);
    try {
      await waitForDriver(tauriDriver);
      const capabilities = new Capabilities()
        .setBrowserName(TAURI_BROWSER_NAME)
        .set('tauri:options', { application });
      const driver = await new Builder()
        .usingServer(DRIVER_URL)
        .withCapabilities(capabilities)
        .build();
      return new DesktopSession(driver, folders, tauriDriver);
    } catch (error) {
      await release(tauriDriver, folders);
      throw error;
    }
  }

  /** Closes the app, stops tauri-driver and removes the temporary folders. */
  async stop(): Promise<void> {
    try {
      await this.driver.quit();
    } finally {
      await release(this.#tauriDriver, this.folders);
    }
  }
}

/** Stops tauri-driver, then removes the folders; a folder left behind is only a warning. */
async function release(tauriDriver: ChildProcess, folders: SessionFolders): Promise<void> {
  if (tauriDriver.exitCode === null && tauriDriver.signalCode === null) {
    const exited = once(tauriDriver, 'exit');
    tauriDriver.kill();
    await exited;
  }
  const removal = rm(folders.root, { recursive: true, force: true, maxRetries: REMOVAL_RETRIES });
  await removal.catch((error: unknown) => {
    console.warn(`${folders.root} was left behind: ${String(error)}`);
  });
}

async function temporaryFolders(): Promise<SessionFolders> {
  const root = await mkdtemp(path.join(os.tmpdir(), 'life-pixel-e2e-'));
  return {
    root,
    library: path.join(root, 'library'),
    exports: path.join(root, 'exports'),
    home: path.join(root, 'home'),
  };
}

/** tauri-driver, whose native driver starts the app with this environment. */
function startTauriDriver(folders: SessionFolders): ChildProcess {
  const command = process.env['TAURI_DRIVER'] ?? 'tauri-driver';
  const args = ['--port', DRIVER_PORT, '--native-port', NATIVE_DRIVER_PORT];
  const env = {
    ...process.env,
    LIFE_PIXEL_LIBRARY: folders.library,
    LP_EXPORT_DIR: folders.exports,
    XDG_CONFIG_HOME: path.join(folders.home, 'config'),
    XDG_DATA_HOME: path.join(folders.home, 'data'),
    XDG_CACHE_HOME: path.join(folders.home, 'cache'),
    LANG: LOCALE,
    LC_ALL: LOCALE,
  };
  return spawn(command, args, { env, stdio: 'inherit' });
}

/** Resolves once tauri-driver answers; fails if it exits or stays silent. */
async function waitForDriver(tauriDriver: ChildProcess): Promise<void> {
  const deadline = Date.now() + DRIVER_START_TIMEOUT_MS;
  while (Date.now() < deadline) {
    if (tauriDriver.exitCode !== null) {
      throw new Error(`tauri-driver exited with code ${String(tauriDriver.exitCode)}`);
    }
    const answered = await fetch(`${DRIVER_URL}status`).then(
      () => true,
      () => false,
    );
    if (answered) return;
    await sleep(DRIVER_POLL_INTERVAL_MS);
  }
  throw new Error(
    `tauri-driver did not answer on ${DRIVER_URL} within ${String(DRIVER_START_TIMEOUT_MS)} ms`,
  );
}
