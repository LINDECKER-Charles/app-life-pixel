import type { AccessTokenScope, CreatedAccessToken } from 'shared';

/**
 * `core::limits`, transcribed for the token form (mcp-cli.md, A3): like `ACCOUNT_LIMITS`, it spares
 * the tokens page the editor's wasm engine.
 */
export const TOKEN_LIMITS = {
  nameMaxChars: 60,
  expiryDays: [30, 90, 365],
  defaultExpiryDays: 90,
} as const;

/** The scopes in the order the form lists them. */
export const SCOPES: readonly AccessTokenScope[] = ['read', 'write', 'export'];

/** What a new token grants unless the person changes it. */
export const DEFAULT_SCOPES: readonly AccessTokenScope[] = ['read', 'write'];

// Literal keys, so that the i18n check sees each one used.
/** The label of each scope. */
export const SCOPE_LABELS: Readonly<Record<AccessTokenScope, string>> = {
  read: 'tokens.scope.read',
  write: 'tokens.scope.write',
  export: 'tokens.scope.export',
};

/** What each scope lets an agent do, under its label. */
export const SCOPE_HINTS: Readonly<Record<AccessTokenScope, string>> = {
  read: 'tokens.scope.read_hint',
  write: 'tokens.scope.write_hint',
  export: 'tokens.scope.export_hint',
};

/** The characters of `name` once trimmed, as the server counts them: code points. */
export function nameLength(name: string): number {
  return [...name.trim()].length;
}

/** Whether the server accepts `name`: 1 to `nameMaxChars` characters once trimmed. */
export function isValidName(name: string): boolean {
  const length = nameLength(name);
  return length > 0 && length <= TOKEN_LIMITS.nameMaxChars;
}

/** The command that registers the endpoint in Claude Code with the secret of `created`. */
export function mcpCommand(created: CreatedAccessToken): string {
  const { serverName, url } = created.mcp;
  return (
    `claude mcp add --transport http ${serverName} ${url} ` +
    `--header "Authorization: Bearer ${created.token}"`
  );
}

/** `iso` as a date of `language`. */
export function formatDate(iso: string, language: string): string {
  return new Intl.DateTimeFormat(language, { dateStyle: 'medium' }).format(new Date(iso));
}
