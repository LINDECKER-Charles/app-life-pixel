import { ChangeDetectionStrategy, Component } from '@angular/core';

/**
 * The editor page's canvas: an empty stub, filled by U2 (editor.md). It takes no input: the
 * feature reads EditorStore and EngineStore.
 */
@Component({
  selector: 'lp-canvas',
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: '',
})
export class Canvas {}
