import { loadExport } from './module-cache';
import { FRAME_CHANGED, PlayerInstance, RANGE_ENDED } from './player-instance';

/** The longest time one animation frame passes to `tick`, after a stall. */
const LONGEST_TICK_MS = 1000;
const REDUCED_MOTION = '(prefers-reduced-motion: reduce)';
const STYLE =
  ':host{display:inline-block}' +
  'canvas{display:block;width:100%;height:auto;image-rendering:pixelated}';
/** `set_loop`'s modes, for the `loop` attribute absent, on, and `false`. */
const LOOP_OWN_MODE = 0;
const LOOP_ALWAYS = 1;
const LOOP_ONCE = 2;

/** `<life-pixel>`: plays a Life Pixel export on a canvas at the animation's native size. */
export class LifePixelElement extends HTMLElement {
  static readonly observedAttributes = ['src', 'tag', 'loop', 'alt'];
  readonly #canvas = document.createElement('canvas');
  #player?: PlayerInstance;
  #observer?: IntersectionObserver;
  #isPlaying = false;
  #hasEnded = false;
  #isVisible = false;
  #request = 0;
  #lastTime?: number;

  constructor() {
    super();
    const root = this.attachShadow({ mode: 'open' });
    root.innerHTML = `<style>${STYLE}</style>`;
    root.append(this.#canvas);
    this.#clear();
  }

  get src(): string {
    return this.getAttribute('src') ?? '';
  }

  set src(value: string) {
    this.setAttribute('src', value);
  }

  get tag(): string {
    return this.getAttribute('tag') ?? '';
  }

  set tag(value: string) {
    this.setAttribute('tag', value);
  }

  /** Whether the animation plays: started, not paused, and not stopped at the end of its range. */
  get playing(): boolean {
    return this.#isPlaying && !this.#hasEnded;
  }

  play(): void {
    this.#isPlaying = true;
    // A range played once to its end starts again from its first frame.
    if (this.#hasEnded) this.seek(0);
    this.#update();
  }

  pause(): void {
    this.#isPlaying = false;
    this.#update();
  }

  /** Shows a frame of the current range, counted from its first frame; ignored if it has none. */
  seek(frame: number): void {
    if (this.#player?.seek(frame)) this.#restart();
  }

  connectedCallback(): void {
    this.setAttribute('role', 'img');
    this.#label();
    this.#observer = new IntersectionObserver((entries) => {
      for (const entry of entries) this.#isVisible = entry.isIntersecting;
      this.#update();
    });
    this.#observer.observe(this);
    document.addEventListener('visibilitychange', this.#update);
  }

  disconnectedCallback(): void {
    this.#observer?.disconnect();
    document.removeEventListener('visibilitychange', this.#update);
    this.#isVisible = false;
    this.#update();
  }

  attributeChangedCallback(name: string): void {
    if (name === 'src') void this.#load();
    else if (name === 'tag') this.#showTag();
    else if (name === 'loop') this.#setLoop();
    else this.#label();
  }

  async #load(): Promise<void> {
    const src = this.src;
    this.#player = undefined;
    this.#clear();
    this.#update();
    if (!src) return;
    try {
      const compiled = await loadExport(new URL(src, document.baseURI).href);
      const player = await PlayerInstance.create(compiled);
      if (src === this.src) this.#start(player);
    } catch (error) {
      if (src !== this.src) return;
      console.warn(`<life-pixel> ${src}: ${error}`);
      this.dispatchEvent(new Event('error'));
    }
  }

  #start(player: PlayerInstance): void {
    this.#player = player;
    this.#canvas.width = player.width;
    this.#canvas.height = player.height;
    this.#setLoop();
    this.#showTag();
    this.#label();
    const isMotionReduced =
      this.getAttribute('motion') !== 'always' && matchMedia(REDUCED_MOTION).matches;
    this.#isPlaying = this.getAttribute('autoplay') !== 'false' && !isMotionReduced;
    this.dispatchEvent(new Event('load'));
    this.#update();
  }

  #showTag(): void {
    const player = this.#player;
    if (!player) return;
    const tag = this.tag;
    const index = player.tags.indexOf(tag);
    if (tag && index < 0) console.warn(`<life-pixel> unknown tag: ${tag}`);
    player.showTag(index);
    this.#restart();
  }

  #setLoop(): void {
    const loop = this.getAttribute('loop');
    const mode = loop === null ? LOOP_OWN_MODE : loop === 'false' ? LOOP_ONCE : LOOP_ALWAYS;
    this.#player?.setLoop(mode);
  }

  #label(): void {
    const alt = this.getAttribute('alt');
    const label = alt ?? this.#player?.title;
    if (alt === '') this.setAttribute('aria-hidden', 'true');
    else this.removeAttribute('aria-hidden');
    if (label) this.setAttribute('aria-label', label);
    else this.removeAttribute('aria-label');
  }

  /** After a range was shown from its start or sought: draw it, and play it if it had ended. */
  #restart(): void {
    this.#hasEnded = false;
    this.#draw();
    this.#update();
  }

  #draw(): void {
    const player = this.#player;
    if (player) this.#canvas.getContext('2d')?.putImageData(player.frame, 0, 0);
  }

  #clear(): void {
    this.#canvas.width = this.#canvas.height = 0;
  }

  #canRun(): boolean {
    return !!this.#player && this.playing && this.#isVisible && !document.hidden;
  }

  /** Starts or stops the animation frame loop, as the element's state now requires. */
  readonly #update = (): void => {
    if (!this.#canRun()) {
      cancelAnimationFrame(this.#request);
      this.#request = 0;
    } else if (!this.#request) {
      this.#lastTime = undefined;
      this.#request = requestAnimationFrame(this.#step);
    }
  };

  readonly #step = (time: number): void => {
    const player = this.#player;
    if (!player) return;
    // Whole milliseconds reach `tick`; the fraction carries over to the next frame.
    const last = this.#lastTime ?? time;
    const elapsed = Math.min(Math.floor(time - last), LONGEST_TICK_MS);
    this.#lastTime = elapsed < LONGEST_TICK_MS ? last + elapsed : time;
    const flags = player.tick(elapsed);
    if (flags & FRAME_CHANGED) this.#draw();
    if (flags & RANGE_ENDED) {
      this.#hasEnded = player.hasStopped();
      this.dispatchEvent(new Event('tagend'));
    }
    this.#request = this.#canRun() ? requestAnimationFrame(this.#step) : 0;
  };
}
