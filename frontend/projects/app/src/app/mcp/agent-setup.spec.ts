import { claudeCommand, mcpServersEntry } from './agent-setup';

const PATHS = {
  cliPath: '/Applications/Life Pixel.app/Contents/MacOS/life-pixel',
  libraryPath: '/Users/pixel/Documents/Life Pixel',
};

describe('the agent setup', () => {
  it('adds the bundled CLI to Claude Code by its absolute path, on the library in use', () => {
    expect(claudeCommand(PATHS)).toBe(
      'claude mcp add life-pixel -- "/Applications/Life Pixel.app/Contents/MacOS/life-pixel" ' +
        'mcp --library "/Users/pixel/Documents/Life Pixel"',
    );
  });

  it('gives the same server as an mcpServers entry, Windows paths escaped as JSON', () => {
    expect(JSON.parse(mcpServersEntry(PATHS))).toEqual({
      mcpServers: {
        'life-pixel': { command: PATHS.cliPath, args: ['mcp', '--library', PATHS.libraryPath] },
      },
    });
    const windows = { cliPath: 'C:\\Life Pixel\\life-pixel.exe', libraryPath: 'D:\\Pixels' };
    expect(mcpServersEntry(windows)).toContain('"C:\\\\Life Pixel\\\\life-pixel.exe"');
  });
});
