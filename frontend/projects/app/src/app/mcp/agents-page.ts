import { ChangeDetectionStrategy, Component, computed, inject, signal } from '@angular/core';
import { IonContent } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { PlatformService, type PlatformInfo } from '../platform/platform';
import { claudeCommand, mcpServersEntry } from './agent-setup';
import { CopyBlock } from './copy-block';

/** What the page knows of the platform: still reading, read, or unreadable. */
type InfoLookup =
  | { readonly kind: 'loading' }
  | { readonly kind: 'found'; readonly info: PlatformInfo }
  | { readonly kind: 'failed' };

/** What an agent can do once connected, one i18n key per line. */
const ABILITIES = [
  'mcp.agents.abilities.find',
  'mcp.agents.abilities.draw',
  'mcp.agents.abilities.animate',
  'mcp.agents.abilities.preview',
  'mcp.agents.abilities.export',
] as const;

/**
 * `settings/agents`, on the desktop only (desktop.md, T3): how to connect a local agent to this
 * library through the bundled `life-pixel` CLI — the `claude mcp add` command and the
 * `mcpServers` entry, each with a copy button —, and what the agent can then do.
 */
@Component({
  selector: 'lp-agents-page',
  imports: [CopyBlock, IonContent, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './agents-page.html',
  styleUrl: './agents-page.scss',
})
export class AgentsPage {
  private readonly platform = inject(PlatformService);

  protected readonly abilities = ABILITIES;
  protected readonly lookup = signal<InfoLookup>({ kind: 'loading' });
  /** The paths an agent needs, once the build is known to ship the CLI. */
  protected readonly paths = computed(() => {
    const lookup = this.lookup();
    if (lookup.kind !== 'found' || lookup.info.cliPath === null) return null;
    return { cliPath: lookup.info.cliPath, libraryPath: lookup.info.libraryPath };
  });
  protected readonly command = computed(() => {
    const paths = this.paths();
    return paths === null ? '' : claudeCommand(paths);
  });
  protected readonly configuration = computed(() => {
    const paths = this.paths();
    return paths === null ? '' : mcpServersEntry(paths);
  });

  constructor() {
    void this.readInfo();
  }

  private async readInfo(): Promise<void> {
    try {
      this.lookup.set({ kind: 'found', info: await this.platform.info() });
    } catch {
      this.lookup.set({ kind: 'failed' });
    }
  }
}
