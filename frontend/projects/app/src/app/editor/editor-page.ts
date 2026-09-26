import {
  ChangeDetectionStrategy,
  Component,
  computed,
  effect,
  inject,
  input,
  untracked,
} from '@angular/core';
import type { ViewDidEnter, ViewWillLeave } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { Canvas } from '../canvas/canvas';
import { EngineStore } from '../engine/engine-store';
import { ExportButton } from '../export/export-button';
import { OpenAnimationFlow } from '../library/open/open-animation-flow';
import { OpenStatus } from '../library/open/open-status';
import { SaveButton } from '../library/save/save-button';
import { SaveStatePill } from '../library/save/state/save-state-pill';
import { PalettePanel } from '../palette/palette-panel';
import { LayerList } from '../timeline/layers/layer-list';
import { PlaybackPreview } from '../timeline/playback/playback-preview';
import { Timeline } from '../timeline/timeline';
import { ToolBar } from '../tools/tool-bar';
import { Icon } from '../ui/icon/icon';
import { NewAnimationDialog } from './new-animation/new-animation-dialog';
import { NewAnimationFlow } from './new-animation/new-animation-flow';
import { Shortcuts } from './shortcuts';
import { AnimationTitle } from './title/animation-title';
import { EditorWelcome } from './welcome/editor-welcome';

/**
 * The editor (plan C7): a document bar — the title, the save state, New, Save and Export —, the
 * tool rail, the canvas stage, the inspector — palette, layers, playback preview — and the
 * timeline. With no document — a first visit —, the stage shows the welcome, whose action opens
 * the new-animation dialog (C10); with an animation id, it opens that saved animation (H8). It
 * hands key presses to the shortcuts while it is the page shown.
 */
@Component({
  selector: 'lp-editor-page',
  imports: [
    AnimationTitle,
    Canvas,
    EditorWelcome,
    ExportButton,
    Icon,
    LayerList,
    NewAnimationDialog,
    OpenStatus,
    PalettePanel,
    PlaybackPreview,
    SaveButton,
    SaveStatePill,
    Timeline,
    ToolBar,
    TranslocoPipe,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './editor-page.html',
  styleUrl: './editor-page.scss',
  host: { '(document:keydown)': 'onKeydown($event)' },
})
export class EditorPage implements ViewDidEnter, ViewWillLeave {
  private readonly engine = inject(EngineStore);
  private readonly shortcuts = inject(Shortcuts);
  private readonly openAnimation = inject(OpenAnimationFlow);
  private isShown = true;

  protected readonly newAnimation = inject(NewAnimationFlow);

  /** From the route `editor/:animationId`: a saved animation to open (H8), so no welcome. */
  readonly animationId = input<string>();

  /** The engine is up with nothing open, and no saved animation is on its way. */
  protected readonly showWelcome = computed(
    () => this.engine.state().status === 'empty' && this.animationId() === undefined,
  );

  constructor() {
    effect(() => {
      const id = this.animationId();
      if (id !== undefined) untracked(() => void this.openAnimation.open(id));
    });
  }

  ionViewDidEnter(): void {
    this.isShown = true;
  }

  ionViewWillLeave(): void {
    this.isShown = false;
  }

  protected onKeydown(event: KeyboardEvent): void {
    if (this.isShown) this.shortcuts.handle(event);
  }
}
