import { error, Key, type WebDriver, type WebElement } from 'selenium-webdriver';
import { ROLE_QUERY_SCRIPT } from './role-query.ts';

/** How long an element may take to appear, to go, or to take an action. */
const DEFAULT_TIMEOUT_MS = 20_000;

/** The accessible name wanted: its exact text, or a pattern; any name when omitted. */
export type Name = string | RegExp;

/** Where to look, and for how long. */
export interface Options {
  readonly within?: WebElement;
  readonly timeoutMs?: number;
}

/**
 * The app's window as a person finds it: every element by its role and accessible name, among
 * those shown — not behind an open dialog, nor on a page Ionic keeps hidden.
 */
export class Screen {
  readonly #driver: WebDriver;

  constructor(driver: WebDriver) {
    this.#driver = driver;
  }

  /** The shown elements of `role` named `name`, now. */
  queryAll(role: string, name?: Name, within?: WebElement): Promise<WebElement[]> {
    return this.#driver.executeScript<WebElement[]>(
      ROLE_QUERY_SCRIPT,
      within ?? null,
      role,
      toMatch(name),
    );
  }

  /** The first shown element of `role` named `name`, once there is one. */
  async find(role: string, name?: Name, options: Options = {}): Promise<WebElement> {
    const problem = `no ${role} named ${describe(name)} was shown`;
    const found = await this.#driver.wait(
      async () => (await this.queryAll(role, name, options.within))[0] ?? null,
      options.timeoutMs ?? DEFAULT_TIMEOUT_MS,
      problem,
    );
    if (found === null) throw new Error(problem);
    return found;
  }

  /**
   * Runs `action` on the only shown element of `role` named `name`, once it succeeds: while Ionic
   * moves between two pages, both are shown, and a closing dialog may still hold the focus.
   */
  async act(
    role: string,
    name: Name,
    action: (element: WebElement) => Promise<void>,
  ): Promise<WebElement> {
    let state = 'none was shown';
    const attempt = async (): Promise<WebElement | null> => {
      const shown = await this.queryAll(role, name);
      const [element] = shown;
      if (element === undefined || shown.length > 1) {
        state = `${String(shown.length)} were shown`;
        return null;
      }
      try {
        await action(element);
        return element;
      } catch (failure) {
        if (!isTransient(failure)) throw failure;
        state = String(failure);
        return null;
      }
    };
    const used = await this.#driver.wait(attempt, DEFAULT_TIMEOUT_MS).catch((failure: unknown) => {
      if (!(failure instanceof error.TimeoutError)) throw failure;
      throw new Error(`the only ${role} named ${describe(name)} could not be used: ${state}`);
    });
    if (used === null) throw new Error(`no ${role} named ${describe(name)}`);
    return used;
  }

  /**
   * Replaces the value of the only shown field of `role` named `name` by typing, as a person does
   * after selecting it all — again when a dialog still opening took the keys.
   */
  async type(role: string, name: Name, value: string): Promise<void> {
    await this.act(role, name, async (field) => {
      // Two calls: WebKitGTK's WebDriver keeps Ctrl down through the chord's release in one.
      await field.sendKeys(Key.chord(Key.CONTROL, 'a'));
      await field.sendKeys(value);
      if ((await field.getProperty('value')) !== value) {
        throw new error.ElementNotInteractableError(`“${value}” did not reach the field`);
      }
    });
  }

  /** Resolves once no element of `role` named `name` is shown. */
  async waitUntilGone(role: string, name?: Name, options: Options = {}): Promise<void> {
    await this.#driver.wait(
      async () => (await this.queryAll(role, name, options.within)).length === 0,
      options.timeoutMs ?? DEFAULT_TIMEOUT_MS,
      `a ${role} named ${describe(name)} was still shown`,
    );
  }

  /** Resolves once `element` is enabled. */
  async waitUntilEnabled(element: WebElement): Promise<void> {
    await this.#driver.wait(() => element.isEnabled(), DEFAULT_TIMEOUT_MS, 'still disabled');
  }

  /** Resolves once `element`'s `aria-pressed` is `true`. */
  async waitUntilPressed(element: WebElement): Promise<void> {
    await this.#driver.wait(
      async () => (await element.getAttribute('aria-pressed')) === 'true',
      DEFAULT_TIMEOUT_MS,
      'still not pressed',
    );
  }
}

/** A failure that goes once the window settles: the element is covered, moving, or replaced. */
function isTransient(failure: unknown): boolean {
  return (
    failure instanceof error.ElementNotInteractableError ||
    failure instanceof error.ElementClickInterceptedError ||
    failure instanceof error.StaleElementReferenceError
  );
}

function toMatch(name: Name | undefined): object | null {
  if (name === undefined) return null;
  if (typeof name === 'string') return { exact: name };
  return { pattern: name.source, flags: name.flags };
}

function describe(name: Name | undefined): string {
  return name === undefined ? 'anything' : `“${String(name)}”`;
}
