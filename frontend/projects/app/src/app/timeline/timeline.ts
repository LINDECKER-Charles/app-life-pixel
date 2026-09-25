import { ChangeDetectionStrategy, Component } from '@angular/core';

/**
 * The editor page's timeline: an empty stub, filled by U3 (editor.md). It takes no input: the
 * feature reads EditorStore and EngineStore.
 */
@Component({
  selector: 'lp-timeline',
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: '',
})
export class Timeline {}
