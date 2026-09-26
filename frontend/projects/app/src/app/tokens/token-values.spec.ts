import { CREATED } from './testing/tokens-test-support';
import { isValidName, mcpCommand, TOKEN_LIMITS } from './token-values';

describe('token values', () => {
  it('builds the command that registers the endpoint with the secret', () => {
    expect(mcpCommand(CREATED)).toBe(
      'claude mcp add --transport http life-pixel https://life-pixel.app/mcp ' +
        `--header "Authorization: Bearer ${CREATED.token}"`,
    );
  });

  it('accepts a name of 1 to the maximum of characters once trimmed', () => {
    expect(isValidName('  CI  ')).toBe(true);
    expect(isValidName('é'.repeat(TOKEN_LIMITS.nameMaxChars))).toBe(true);
    expect(isValidName('   ')).toBe(false);
    expect(isValidName('n'.repeat(TOKEN_LIMITS.nameMaxChars + 1))).toBe(false);
  });
});
