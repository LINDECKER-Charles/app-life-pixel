import '@life-pixel/player';

import {
  ChangeDetectionStrategy,
  Component,
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

/** The tag whose range covers `frame`, or `undefined` to play every frame (editor.md, U3). */
function activeTagName(document: DocumentSummary, frame: FrameId | null): string | undefined {
  const index = document.frames.findIndex((candidate) => candidate.id === frame);
  if (index < 0) return undefined;
  return document.tags.find((tag) => index >= tag.first && index <= tag.last)?.name;
}

/**
 * Plays a fresh WASM export of the animation in a `<life-pixel>` element, from a `blob:` URL, on
 * the tag holding the active frame or on every frame. It never starts on its own — only `P`, or
 * the button, starts it —, and any later engine change stops it (editor.md, U3).
 */
@Component({
  selector: 'lp-playback-preview',
  imports: [TranslocoPipe],
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
    if (this.src() !== null) {
      this.stop();
      return;
    }
    await this.play();
  }

  private async play(): Promise<void> {
    const document = this.engineStore.document();
    if (!document) return;
    const tagName = activeTagName(document, this.editor.activeFrame());
    const result = await this.engine.export({ format: 'wasm', tag: tagName }).catch(() => null);
    const file = result?.files.find((candidate) => candidate.mediaType === 'application/wasm');
    if (!file) return;
    this.revoke();
    this.blobUrl = URL.createObjectURL(
      new Blob([new Uint8Array(file.bytes)], { type: file.mediaType }),
    );
    this.src.set(this.blobUrl);
    this.tag.set(tagName ?? '');
    this.startedState.set(this.engineStore.state());
  }

  protected stop(): void {
    this.src.set(null);
    this.tag.set('');
    this.startedState.set(null);
    this.revoke();
  }

  private revoke(): void {
    if (this.blobUrl === null) return;
    URL.revokeObjectURL(this.blobUrl);
    this.blobUrl = null;
  }
}
