import type { EditorTool } from '../editor/editor-store';

export interface ToolDefinition {
  readonly tool: EditorTool;
  readonly labelKey: string;
  /** `KeyboardEvent.key`, as shown in the tooltip and used to register the shortcut. */
  readonly shortcutKey: string;
}

/** The tool bar's buttons, in order, with the shortcut of editor.md's U4 table. */
export const TOOL_DEFINITIONS: readonly ToolDefinition[] = [
  { tool: 'pencil', labelKey: 'tools.tool.pencil', shortcutKey: 'b' },
  { tool: 'eraser', labelKey: 'tools.tool.eraser', shortcutKey: 'e' },
  { tool: 'fill', labelKey: 'tools.tool.fill', shortcutKey: 'g' },
  { tool: 'line', labelKey: 'tools.tool.line', shortcutKey: 'l' },
  { tool: 'rectangle', labelKey: 'tools.tool.rectangle', shortcutKey: 'r' },
  { tool: 'select', labelKey: 'tools.tool.select', shortcutKey: 'm' },
];
