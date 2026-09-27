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
export function claudeCommand(paths: AgentPaths): string {
  return `claude mcp add ${SERVER_NAME} ${launchArguments(paths)}`;
}

/** The same server added to Codex, whose `mcp add` takes the same arguments. */
export function codexCommand(paths: AgentPaths): string {
  return `codex mcp add ${SERVER_NAME} ${launchArguments(paths)}`;
}

/** The same server, as the `mcpServers` entry of an MCP client's JSON configuration. */
export function mcpServersEntry({ cliPath, libraryPath }: AgentPaths): string {
  const server = { command: cliPath, args: ['mcp', '--library', libraryPath] };
  return JSON.stringify({ mcpServers: { [SERVER_NAME]: server } }, null, 2);
}

/** What follows `--` in an `mcp add` command: how the client launches the server. */
function launchArguments({ cliPath, libraryPath }: AgentPaths): string {
  return `-- "${cliPath}" mcp --library "${libraryPath}"`;
}
