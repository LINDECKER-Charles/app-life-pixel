import { execFile } from 'node:child_process';
import { promisify } from 'node:util';

/** An animation as `life-pixel list --json` prints it. */
export interface ListedAnimation {
  readonly id: string;
  readonly title: string;
  readonly projectId: string;
  readonly width: number;
  readonly height: number;
  readonly frameCount: number;
  readonly updatedAt: string;
}

/** The animations of `library`, as the bundled CLI at `cli` lists them. */
export async function cliAnimations(cli: string, library: string): Promise<ListedAnimation[]> {
  const { stdout } = await promisify(execFile)(cli, ['list', '--library', library, '--json']);
  return JSON.parse(stdout) as ListedAnimation[];
}
