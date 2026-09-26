/** The name agents know the local server by: the tools appear as `life-pixel`'s. */
export const SERVER_NAME = 'life-pixel';

/** Where the bundled CLI and the library are, as `platform_info` gives them. */
export interface AgentPaths {
  readonly cliPath: string;
  readonly libraryPath: string;
}

/**
 * The command that adds the desktop's library to Claude Code as a local MCP server (desktop.md,
 * T3): the bundled CLI by its absolute path, on the library folder in use.
 */
export function claudeCommand({ cliPath, libraryPath }: AgentPaths): string {
  return `claude mcp add ${SERVER_NAME} -- "${cliPath}" mcp --library "${libraryPath}"`;
}

/** The same server, as the `mcpServers` entry of an MCP client's JSON configuration. */
export function mcpServersEntry({ cliPath, libraryPath }: AgentPaths): string {
  const server = { command: cliPath, args: ['mcp', '--library', libraryPath] };
  return JSON.stringify({ mcpServers: { [SERVER_NAME]: server } }, null, 2);
}
