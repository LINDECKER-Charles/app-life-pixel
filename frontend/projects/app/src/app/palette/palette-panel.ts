import { ChangeDetectionStrategy, Component } from '@angular/core';

/**
 * The editor page's palette panel: an empty stub, filled by U4 (editor.md). It takes no input: the
 * feature reads EditorStore and EngineStore.
 */
@Component({
  selector: 'lp-palette-panel',
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: '',
})
export class PalettePanel {}
