import { CREATED } from './testing/tokens-test-support';
import {
  claudeHttpCommand,
  codexHttpCommand,
  isValidName,
  TOKEN_LIMITS,
  tokenVariable,
} from './token-values';

describe('token values', () => {
  it('builds the Claude Code command that registers the endpoint with the secret', () => {
    expect(claudeHttpCommand(CREATED)).toBe(
      'claude mcp add --transport http life-pixel https://life-pixel.app/mcp ' +
        `--header "Authorization: Bearer ${CREATED.token}"`,
    );
  });

  it('builds the Codex command, which reads the secret from a variable named after the server', () => {
    expect(codexHttpCommand(CREATED)).toBe(
      'codex mcp add life-pixel --url https://life-pixel.app/mcp ' +
        '--bearer-token-env-var LIFE_PIXEL_TOKEN',
    );
    expect(codexHttpCommand(CREATED)).not.toContain(CREATED.token);
    expect(tokenVariable('life-pixel-staging')).toBe('LIFE_PIXEL_STAGING_TOKEN');
  });

  it('accepts a name of 1 to the maximum of characters once trimmed', () => {
    expect(isValidName('  CI  ')).toBe(true);
    expect(isValidName('é'.repeat(TOKEN_LIMITS.nameMaxChars))).toBe(true);
    expect(isValidName('   ')).toBe(false);
    expect(isValidName('n'.repeat(TOKEN_LIMITS.nameMaxChars + 1))).toBe(false);
  });
});
