/**
 * `<life-pixel>`: plays a Life Pixel export — a `.wasm` file — on a canvas at the animation's
 * native size, scaled pixel-exact by CSS.
 *
 * Attributes: `src`, `tag`, `autoplay` (`"false"` waits for `play()`), `loop` (present or
 * `"true"` loops, `"false"` plays once), `alt` (`""` marks the element decorative) and `motion`
 * (`"always"` plays even when reduced motion is preferred).
 */
export declare class LifePixelElement extends HTMLElement {
  /** The export's URL; reflects the `src` attribute. Changing it loads the new export. */
  src: string;
  /** The tag played, by name; reflects the `tag` attribute. Empty: the first tag. */
  tag: string;
  /** Whether the animation plays: started, not paused, and not stopped at the end of its range. */
  readonly playing: boolean;
  /** Plays the animation, from the start of its range when it had played once to its end. */
  play(): void;
  /** Pauses the animation on the frame shown. */
  pause(): void;
  /** Shows a frame of the current range, counted from its first frame; ignored if it has none. */
  seek(frame: number): void;

  addEventListener<K extends keyof LifePixelElementEventMap>(
    type: K,
    listener: (this: LifePixelElement, event: LifePixelElementEventMap[K]) => unknown,
    options?: boolean | AddEventListenerOptions,
  ): void;
  addEventListener(
    type: string,
    listener: EventListenerOrEventListenerObject,
    options?: boolean | AddEventListenerOptions,
  ): void;
  removeEventListener<K extends keyof LifePixelElementEventMap>(
    type: K,
    listener: (this: LifePixelElement, event: LifePixelElementEventMap[K]) => unknown,
    options?: boolean | EventListenerOptions,
  ): void;
  removeEventListener(
    type: string,
    listener: EventListenerOrEventListenerObject,
    options?: boolean | EventListenerOptions,
  ): void;
}

/** The events of `<life-pixel>`, none of which bubbles; `error` is a plain `Event`. */
export interface LifePixelElementEventMap extends Omit<HTMLElementEventMap, 'error'> {
  /** The first frame is drawn. */
  load: Event;
  /** The export could not be played; a console warning says why. */
  error: Event;
  /** The range reached its end: once when played once, on every loop otherwise. */
  tagend: Event;
}

declare global {
  interface HTMLElementTagNameMap {
    'life-pixel': LifePixelElement;
  }
}
