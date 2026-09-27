import { ChangeDetectionStrategy, Component, computed, inject, signal } from '@angular/core';
import { RouterLink } from '@angular/router';
import { IonContent } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { PlatformService, type PlatformInfo } from '../platform/platform';
import { Icon } from '../ui/icon/icon';
import { StatusBanner } from '../ui/status-banner/status-banner';
import { claudeCommand, codexCommand, mcpServersEntry, type AgentPaths } from './agent-setup';
import { CopyBlock } from './copy-block';

/** What the page knows of the platform: still reading, read, or unreadable. */
type InfoLookup =
  | { readonly kind: 'loading' }
  | { readonly kind: 'found'; readonly info: PlatformInfo }
  | { readonly kind: 'failed' };

/** One way to register the server, shown as a card: i18n keys, then the text to copy. */
interface AgentSetup {
  readonly id: string;
  readonly heading: string;
  readonly hint: string;
  readonly copyLabel: string;
  readonly text: string;
}

/** What an agent can do once connected, one i18n key per line. */
const ABILITIES = [
  'mcp.agents.abilities.find',
  'mcp.agents.abilities.draw',
  'mcp.agents.abilities.animate',
  'mcp.agents.abilities.preview',
  'mcp.agents.abilities.export',
] as const;

// Literal keys, so that the i18n check sees each one used.
/** The ways to register the server: each client's command, then the JSON for the others. */
function agentSetups(paths: AgentPaths): readonly AgentSetup[] {
  return [
    {
      id: 'claude',
      heading: 'mcp.agents.claude.heading',
      hint: 'mcp.agents.command_hint',
      copyLabel: 'mcp.agents.claude.copy',
      text: claudeCommand(paths),
    },
    {
      id: 'codex',
      heading: 'mcp.agents.codex.heading',
      hint: 'mcp.agents.command_hint',
      copyLabel: 'mcp.agents.codex.copy',
      text: codexCommand(paths),
    },
    {
      id: 'json',
      heading: 'mcp.agents.json.heading',
      hint: 'mcp.agents.json.hint',
      copyLabel: 'mcp.agents.json.copy',
      text: mcpServersEntry(paths),
    },
  ];
}

/**
 * `settings/agents`, on the desktop only (desktop.md, T3): how to connect a local agent to this
 * library through the bundled `life-pixel` CLI — the `claude mcp add` and `codex mcp add`
 * commands and the `mcpServers` entry, each with a copy button —, and what the agent can then do.
 */
@Component({
  selector: 'lp-agents-page',
  imports: [CopyBlock, Icon, IonContent, RouterLink, StatusBanner, TranslocoPipe],
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
  protected readonly setups = computed(() => {
    const paths = this.paths();
    return paths === null ? [] : agentSetups(paths);
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
