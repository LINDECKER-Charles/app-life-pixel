import { ChangeDetectionStrategy, Component, inject, signal } from '@angular/core';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { type AccessTokenSummary, ApiProblem, type CreatedAccessToken, TokensApi } from 'shared';
import { TokenCreationDialog } from './token-creation-dialog';
import { TokenRevocation } from './token-revocation';
import { formatDate, SCOPE_LABELS } from './token-values';

/**
 * Settings → Access tokens (mcp-cli.md, A3): the account's active tokens for the hosted MCP
 * endpoint, a new one, and their revocation. `hostedOnly` and `requireAccount` keep this page for
 * a signed-in visitor of the hosted app.
 */
@Component({
  selector: 'lp-tokens-page',
  imports: [TokenCreationDialog, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './tokens-page.html',
  styleUrl: './tokens-page.scss',
})
export class TokensPage {
  private readonly api = inject(TokensApi);
  private readonly revocation = inject(TokenRevocation);
  private readonly transloco = inject(TranslocoService);

  protected readonly scopeLabels = SCOPE_LABELS;

  protected readonly tokens = signal<readonly AccessTokenSummary[]>([]);
  protected readonly loaded = signal(false);
  protected readonly loading = signal(false);
  protected readonly loadError = signal<ApiProblem | undefined>(undefined);
  protected readonly creating = signal(false);
  protected readonly revoking = signal<string | undefined>(undefined);
  protected readonly revokeError = signal<ApiProblem | undefined>(undefined);

  constructor() {
    void this.load();
  }

  protected date(iso: string): string {
    return formatDate(iso, this.transloco.getActiveLang());
  }

  /** Shows a token just created first: it is the most recent. */
  protected onCreated(created: CreatedAccessToken): void {
    const { id, name, prefix, scopes, createdAt, expiresAt, lastUsedAt } = created;
    const summary = { id, name, prefix, scopes, createdAt, expiresAt, lastUsedAt };
    this.tokens.update((tokens) => [summary, ...tokens]);
  }

  protected async load(): Promise<void> {
    this.loading.set(true);
    this.loadError.set(undefined);
    try {
      this.tokens.set(await this.api.list());
      this.loaded.set(true);
    } catch (error) {
      this.loadError.set(problem(error));
    } finally {
      this.loading.set(false);
    }
  }

  /** Revokes `token` once the person confirms it. */
  protected async revoke(token: AccessTokenSummary): Promise<void> {
    if (this.revoking() || !(await this.revocation.confirm(token.name))) return;
    this.revoking.set(token.id);
    this.revokeError.set(undefined);
    try {
      await this.api.revoke(token.id);
      this.tokens.update((tokens) => tokens.filter((candidate) => candidate.id !== token.id));
    } catch (error) {
      this.revokeError.set(problem(error));
    } finally {
      this.revoking.set(undefined);
    }
  }
}

/** `error` as a problem to show; anything but an `ApiProblem` is rethrown. */
function problem(error: unknown): ApiProblem {
  if (!(error instanceof ApiProblem)) throw error;
  return error;
}
