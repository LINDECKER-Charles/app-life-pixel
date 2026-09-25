import { ChangeDetectionStrategy, Component } from '@angular/core';

/**
 * The editor page's export button: an empty stub, filled by U5 (editor.md). It takes no input: the
 * feature reads EditorStore and EngineStore.
 */
@Component({
  selector: 'lp-export-button',
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: '',
})
export class ExportButton {}
