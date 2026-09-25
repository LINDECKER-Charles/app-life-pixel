import {
  ChangeDetectionStrategy,
  Component,
  effect,
  inject,
  input,
  untracked,
} from '@angular/core';
import { IonButton, type ViewDidEnter, type ViewWillLeave } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { Canvas } from '../canvas/canvas';
import { EngineStore } from '../engine/engine-store';
import { ExportButton } from '../export/export-button';
import { PalettePanel } from '../palette/palette-panel';
import { Timeline } from '../timeline/timeline';
import { ToolBar } from '../tools/tool-bar';
import { NewAnimationDialog } from './new-animation/new-animation-dialog';
import { NewAnimationFlow } from './new-animation/new-animation-flow';
import { Shortcuts } from './shortcuts';
import { AnimationTitle } from './title/animation-title';

/**
 * The editor: a header with the title and "New", and the regions U2 to U5 fill. With no document
 * — a first visit —, it opens the new-animation dialog; it hands key presses to the shortcuts
 * while it is the page shown.
 */
@Component({
  selector: 'lp-editor-page',
  imports: [
    AnimationTitle,
    Canvas,
    ExportButton,
    IonButton,
    NewAnimationDialog,
    PalettePanel,
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
  private isShown = true;

  protected readonly newAnimation = inject(NewAnimationFlow);

  /** From the route `editor/:animationId`: a saved animation to open (H8), so no dialog. */
  readonly animationId = input<string>();

  constructor() {
    effect(() => {
      if (this.engine.state().status !== 'empty' || this.animationId() !== undefined) return;
      untracked(() => void this.newAnimation.start());
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
