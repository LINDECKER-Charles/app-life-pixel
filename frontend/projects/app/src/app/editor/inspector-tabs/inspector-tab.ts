/**
 * One tab of the inspector: the panel it shows and its label. Its own element's id is the panel's
 * followed by `-tab`, so that the panel can name itself after it (`aria-labelledby`).
 */
export interface InspectorTab {
  /** The id of the `tabpanel` element this tab controls, from its owner's `idScope`. */
  readonly panelId: string;
  /** The catalogue key of the tab's visible label. */
  readonly labelKey: string;
}
