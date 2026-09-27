import type { EditorTool } from '../editor/editor-store';
import type { IconName } from '../ui/icon/icon-paths';

export interface ToolDefinition {
  readonly tool: EditorTool;
  readonly labelKey: string;
  /** `KeyboardEvent.key`, as shown in the tooltip and used to register the shortcut. */
  readonly shortcutKey: string;
  /** Its button's icon in the tool rail; the translated label names the button. */
  readonly icon: IconName;
}

/** The tool bar's buttons, in order, with the shortcut of editor.md's U4 table. */
export const TOOL_DEFINITIONS: readonly ToolDefinition[] = [
  { tool: 'pencil', labelKey: 'tools.tool.pencil', shortcutKey: 'b', icon: 'pencil' },
  { tool: 'eraser', labelKey: 'tools.tool.eraser', shortcutKey: 'e', icon: 'eraser' },
  { tool: 'fill', labelKey: 'tools.tool.fill', shortcutKey: 'g', icon: 'fill' },
  { tool: 'line', labelKey: 'tools.tool.line', shortcutKey: 'l', icon: 'line' },
  { tool: 'rectangle', labelKey: 'tools.tool.rectangle', shortcutKey: 'r', icon: 'rectangle' },
  { tool: 'select', labelKey: 'tools.tool.select', shortcutKey: 'm', icon: 'select' },
];
