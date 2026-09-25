import { ChangeDetectionStrategy, Component, DestroyRef, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { EditorStore } from '../editor/editor-store';
import { Shortcuts } from '../editor/shortcuts';
import { EngineStore } from '../engine/engine-store';
import { FrameList } from './frames/frame-list';
import { LayerList } from './layers/layer-list';
import { PlaybackPreview } from './playback/playback-preview';
import { TagBars } from './tags/tag-bars';
import { TagDialog } from './tags/tag-dialog';

/**
 * The editor page's timeline (editor.md, U3): the layer list, the frame strip with its tags, and
 * the playback preview. Registers `,` and `.` to step through frames and `O` for onion skin; `P`
 * is `PlaybackPreview`'s own.
 */
@Component({
  selector: 'lp-timeline',
  imports: [FrameList, LayerList, PlaybackPreview, TagBars, TagDialog, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './timeline.html',
  styleUrl: './timeline.scss',
})
export class Timeline {
  private readonly engine = inject(EngineStore);
  private readonly editor = inject(EditorStore);

  constructor() {
    const shortcuts = inject(Shortcuts);
    inject(DestroyRef).onDestroy(
      shortcuts.register([
        { key: ',', label: 'timeline.shortcuts.previous_frame', action: () => this.step(-1) },
        { key: '.', label: 'timeline.shortcuts.next_frame', action: () => this.step(1) },
        { key: 'o', label: 'timeline.shortcuts.onion_skin', action: () => this.toggleOnionSkin() },
      ]),
    );
  }

  private step(direction: 1 | -1): void {
    const frames = this.engine.document()?.frames ?? [];
    const index = frames.findIndex((frame) => frame.id === this.editor.activeFrame());
    const target = index + direction;
    const next = frames[target];
    if (index < 0 || !next) return;
    this.editor.activeFrame.set(next.id);
    this.editor.frameSelection.set({ first: target, last: target });
  }

  private toggleOnionSkin(): void {
    this.editor.onionSkin.update((onionSkin) => ({ ...onionSkin, enabled: !onionSkin.enabled }));
  }
}
