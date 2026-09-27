import '@life-pixel/player';

import {
  ChangeDetectionStrategy,
  Component,
  computed,
  CUSTOM_ELEMENTS_SCHEMA,
  DestroyRef,
  effect,
  inject,
  signal,
} from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { EditorStore } from '../../editor/editor-store';
import { Shortcuts } from '../../editor/shortcuts';
import { EDITOR_ENGINE } from '../../engine/editor-engine';
import { EngineStore } from '../../engine/engine-store';
import type { DocumentSummary, EngineState, FrameId } from '../../engine/engine-types';
import { Icon } from '../../ui/icon/icon';
import { StatusBanner } from '../../ui/status-banner/status-banner';

/**
 * Where the preview stands: waiting for the artist, exporting, loading the export into the
 * player, playing, or failed — the export or the player refused it.
 */
type PlaybackStatus = 'idle' | 'preparing' | 'loading' | 'playing' | 'failed';

/** The tag whose range covers `frame`, or `undefined` to play every frame (editor.md, U3). */
function activeTagName(document: DocumentSummary, frame: FrameId | null): string | undefined {
  const index = document.frames.findIndex((candidate) => candidate.id === frame);
  if (index < 0) return undefined;
  return document.tags.find((tag) => index >= tag.first && index <= tag.last)?.name;
}

/**
 * Plays a fresh WASM export of the animation in a `<life-pixel>` element, from a `blob:` URL, on
 * the tag holding the active frame or on every frame, in a box of fixed height fitted to the
 * inspector; the text under it says which. It never starts on its own — only `P`, or the button,
 * starts it, and then it plays even under reduced motion, as the artist asked for it —, and any
 * later engine change stops it. Preparing and loading show in the box; an export or a player
 * that fails leaves a visible error, never a silent stop (editor.md, U3).
 */
@Component({
  selector: 'lp-playback-preview',
  imports: [Icon, StatusBanner, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  schemas: [CUSTOM_ELEMENTS_SCHEMA],
  templateUrl: './playback-preview.html',
  styleUrl: './playback-preview.scss',
})
export class PlaybackPreview {
  private readonly engineStore = inject(EngineStore);
  private readonly engine = inject(EDITOR_ENGINE);
  private readonly editor = inject(EditorStore);
  private readonly shortcuts = inject(Shortcuts);
  private readonly startedState = signal<EngineState | null>(null);
  private blobUrl: string | null = null;

  protected readonly document = this.engineStore.document;
  protected readonly src = signal<string | null>(null);
  protected readonly tag = signal('');
  protected readonly status = signal<PlaybackStatus>('idle');
  /** The tag a start would play, `undefined` for every frame. */
  protected readonly scopeTag = computed(() => {
    const document = this.document();
    return document ? activeTagName(document, this.editor.activeFrame()) : undefined;
  });

  constructor() {
    const destroyRef = inject(DestroyRef);
    destroyRef.onDestroy(
      this.shortcuts.register([
        { key: 'p', label: 'timeline.shortcuts.play', action: () => void this.toggle() },
      ]),
    );
    destroyRef.onDestroy(() => this.revoke());
    effect(() => {
      const current = this.engineStore.state();
      if (this.startedState() !== null && current !== this.startedState()) this.stop();
    });
  }

  protected async toggle(): Promise<void> {
    if (this.status() === 'preparing') return;
    if (this.src() !== null) {
      this.stop();
      return;
    }
    await this.play();
  }

  /** Exports the animation, then hands the export to the player; a change meanwhile cancels it. */
  private async play(): Promise<void> {
    const document = this.engineStore.document();
    if (!document) return;
    const tagName = this.scopeTag();
    const started = this.engineStore.state();
    this.status.set('preparing');
    this.startedState.set(started);
    const result = await this.engine.export({ format: 'wasm', tag: tagName }).catch(() => null);
    if (this.status() !== 'preparing' || this.startedState() !== started) return;
    const file = result?.files.find((candidate) => candidate.mediaType === 'application/wasm');
    if (!file) {
      this.fail();
      return;
    }
    this.revoke();
    this.blobUrl = URL.createObjectURL(
      new Blob([new Uint8Array(file.bytes)], { type: file.mediaType }),
    );
    this.src.set(this.blobUrl);
    this.tag.set(tagName ?? '');
    this.status.set('loading');
  }

  protected onLoad(): void {
    if (this.src() !== null) this.status.set('playing');
  }

  /** The player could not play the export: a console warning says why, the preview says so. */
  protected onError(): void {
    if (this.src() !== null) this.fail();
  }

  protected stop(): void {
    this.src.set(null);
    this.tag.set('');
    this.startedState.set(null);
    this.status.set('idle');
    this.revoke();
  }

  private fail(): void {
    this.stop();
    this.status.set('failed');
  }

  private revoke(): void {
    if (this.blobUrl === null) return;
    URL.revokeObjectURL(this.blobUrl);
    this.blobUrl = null;
  }
}
