import { inject, Injectable } from '@angular/core';
import { ApiClient } from './api-client';
import type { components } from './schema';

/** What a personal access token grants (mcp-cli.md, A3). */
export type AccessTokenScope = components['schemas']['AccessTokenScope'];
/** A token of the account, without its secret. */
export type AccessTokenSummary = components['schemas']['AccessTokenSummary'];
/** A token just created: `token`, its secret, is in this answer only. */
export type CreatedAccessToken = components['schemas']['CreatedAccessToken'];
/** The MCP server a token is for: what `claude mcp add` registers. */
export type McpServer = components['schemas']['McpServer'];
/** A token to create. */
export type NewAccessToken = components['schemas']['NewAccessToken'];

/**
 * A3's token routes: the signed-in account's personal access tokens for the hosted MCP endpoint,
 * their creation and their revocation.
 */
@Injectable({ providedIn: 'root' })
export class TokensApi {
  private readonly client = inject(ApiClient);

  /** The active tokens, the most recent first. */
  list(): Promise<AccessTokenSummary[]> {
    return this.client.request('get', '/api/v1/tokens');
  }

  /** Creates `token`: `token.name`, `token.expiry` or `token.limit` otherwise. */
  create(token: NewAccessToken): Promise<CreatedAccessToken> {
    return this.client.request('post', '/api/v1/tokens', { body: token });
  }

  /** Revokes the token `id`: `token.not_found` when it is not an active token of the account. */
  async revoke(id: string): Promise<void> {
    await this.client.request('delete', '/api/v1/tokens/{id}', { path: { id } });
  }
}
