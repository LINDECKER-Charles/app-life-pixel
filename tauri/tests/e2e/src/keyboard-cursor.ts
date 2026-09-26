import { Key, type WebElement } from 'selenium-webdriver';

/** A pixel of the animation, from its top-left corner. */
export interface Pixel {
  readonly x: number;
  readonly y: number;
}

/**
 * The canvas's pixel cursor, driven by the arrow keys: it starts at the top-left pixel, and Enter
 * applies the tool where it stands (editor.md, U2).
 */
export class KeyboardCursor {
  readonly #canvas: WebElement;
  #at: Pixel = { x: 0, y: 0 };

  constructor(canvas: WebElement) {
    this.#canvas = canvas;
  }

  /** Moves the cursor to `target`, then presses Enter there. */
  async applyAt(target: Pixel): Promise<void> {
    const dx = target.x - this.#at.x;
    const dy = target.y - this.#at.y;
    await this.#canvas.sendKeys(
      ...repeat(dx < 0 ? Key.ARROW_LEFT : Key.ARROW_RIGHT, Math.abs(dx)),
      ...repeat(dy < 0 ? Key.ARROW_UP : Key.ARROW_DOWN, Math.abs(dy)),
      Key.ENTER,
    );
    this.#at = target;
  }
}

function repeat(key: string, times: number): string[] {
  return Array.from({ length: times }, () => key);
}
