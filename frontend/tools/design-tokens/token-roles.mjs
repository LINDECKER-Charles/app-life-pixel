// The correspondence between the production tokens of
// projects/shared/src/styles/_tokens.scss and the roles of design-system/tokens/tokens.css. The
// production names are older and kept (design-system/docs/adoption.md, "Preserve the token
// contract"): `--lp-color-border` is the design system's control border, and its decorative
// `border` is `--lp-color-border-subtle`.

/** Production custom property → design-system custom property, with the same value. */
export const TOKEN_ROLES = {
  '--lp-color-background': '--lp-bg',
  '--lp-color-surface': '--lp-surface',
  '--lp-color-surface-soft': '--lp-surface-soft',
  '--lp-color-surface-raised': '--lp-surface-raised',
  '--lp-color-text': '--lp-text',
  '--lp-color-text-muted': '--lp-text-muted',
  '--lp-color-border': '--lp-control-border',
  '--lp-color-border-subtle': '--lp-border',
  '--lp-color-accent': '--lp-accent',
  '--lp-color-accent-hover': '--lp-accent-hover',
  '--lp-color-accent-soft': '--lp-accent-soft',
  '--lp-color-accent-text': '--lp-accent-text',
  '--lp-color-on-accent': '--lp-on-accent',
  '--lp-color-focus': '--lp-focus',
  '--lp-color-danger': '--lp-danger',
  '--lp-color-danger-bg': '--lp-danger-bg',
  '--lp-color-success': '--lp-success',
  '--lp-color-success-bg': '--lp-success-bg',
  '--lp-color-warning': '--lp-warning',
  '--lp-color-warning-bg': '--lp-warning-bg',
  '--lp-color-info': '--lp-info',
  '--lp-color-info-bg': '--lp-info-bg',
  '--lp-color-disabled-bg': '--lp-disabled-bg',
  '--lp-color-disabled-text': '--lp-disabled-text',
  '--lp-color-rose': '--lp-rose',
  '--lp-color-peach': '--lp-peach',
  '--lp-color-mint': '--lp-mint',
  '--lp-color-lavender': '--lp-lavender',
  '--lp-color-scrim': '--lp-scrim',
  '--lp-color-canvas-bg': '--lp-canvas-bg',
  '--lp-color-checker-light': '--lp-checker-light',
  '--lp-color-checker-dark': '--lp-checker-dark',
  '--lp-shadow-small': '--lp-shadow-sm',
  '--lp-shadow-medium': '--lp-shadow-md',
  '--lp-shadow-large': '--lp-shadow-lg',
};

/**
 * Production colours with no role in the design system, and why. The solid danger button needs
 * its own foreground (foundations.md): white in light, the dark `on-accent` plum in dark.
 */
export const PRODUCTION_ONLY = {
  '--lp-color-on-danger': 'foreground of the solid danger button (foundations.md, feedback)',
};

/** Colour constants of the canvas renderer, which cannot read custom properties, by role. */
export const RENDERER_CONSTANTS = {
  CHECKER_LIGHT: '--lp-checker-light',
  CHECKER_DARK: '--lp-checker-dark',
  GRID_COLOR: '--lp-canvas-grid',
};
