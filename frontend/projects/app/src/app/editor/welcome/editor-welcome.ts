import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { idScope } from 'shared';
import { LIBRARY_ACCESS } from '../../library/library-access';
import { Icon } from '../../ui/icon/icon';
import { NewAnimationFlow } from '../new-animation/new-animation-flow';
import { viewBarClearance } from './view-bar-clearance';

/**
 * The first screen of an empty editor (plan C10, design-system/docs/journeys.md §1): what can be
 * made, Pip, and one action, "Create animation", which opens the new-animation dialog. A visitor
 * also reads that nothing needs an account, and that unsaved work does not outlive the page (D37).
 * It covers the stage's canvas but not the view bar under it, and scrolls when taller.
 */
@Component({
  selector: 'lp-editor-welcome',
  imports: [Icon, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { '[style.margin-block-end.px]': 'barHeight()' },
  templateUrl: './editor-welcome.html',
  styleUrl: './editor-welcome.scss',
})
export class EditorWelcome {
  protected readonly id = idScope('editor-welcome');
  protected readonly flow = inject(NewAnimationFlow);
  protected readonly access = inject(LIBRARY_ACCESS);
  protected readonly barHeight = viewBarClearance();
}
