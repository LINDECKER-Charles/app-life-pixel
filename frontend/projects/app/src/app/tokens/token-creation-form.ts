import {
  ChangeDetectionStrategy,
  Component,
  inject,
  output,
  signal,
  viewChild,
} from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { type AccessTokenScope, type CreatedAccessToken, idScope, TokensApi } from 'shared';
import { ErrorSummary } from '../account/form/error-summary';
import { fieldError } from '../account/form/field-error';
import { type FieldCheck, type FieldMap, FormErrors } from '../account/form/form-errors';
import { Icon } from '../ui/icon/icon';
import {
  DEFAULT_SCOPES,
  isValidName,
  SCOPE_HINTS,
  SCOPE_LABELS,
  SCOPES,
  TOKEN_LIMITS,
} from './token-values';

type CreationField = 'name' | 'scopes' | 'expiry' | 'form';

const FIELD_BY_CODE: FieldMap<CreationField> = {
  'token.name': 'name',
  'token.expiry': 'expiry',
};

/**
 * A new token's name, scopes — read and write unless changed — and lifetime, 90 days unless
 * changed (mcp-cli.md, A3), checked when sent. Emits the token the server created, its secret in
 * it. Lays out the dialog's body and action row (`.lp-dialog`).
 */
@Component({
  selector: 'lp-token-creation-form',
  imports: [ErrorSummary, Icon, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './token-creation-form.html',
  styleUrl: './token-dialog.scss',
})
export class TokenCreationForm {
  private readonly api = inject(TokensApi);
  private readonly summary = viewChild.required(ErrorSummary);

  /** The token just created. */
  readonly created = output<CreatedAccessToken>();
  /** The person gave up. */
  readonly cancelled = output();

  protected readonly id = idScope('token-creation-form');

  protected readonly limits = TOKEN_LIMITS;
  protected readonly scopeOptions = SCOPES;
  protected readonly scopeLabels = SCOPE_LABELS;
  protected readonly scopeHints = SCOPE_HINTS;
  protected readonly targets = {
    name: this.id('name'),
    scopes: this.id(`scope-${SCOPES[0]}`),
    expiry: this.id('expiry'),
  };

  protected readonly name = signal('');
  protected readonly scopes = signal<ReadonlySet<AccessTokenScope>>(new Set(DEFAULT_SCOPES));
  protected readonly expiryDays = signal<number>(TOKEN_LIMITS.defaultExpiryDays);
  protected readonly pending = signal(false);
  protected readonly errors = new FormErrors<CreationField>(FIELD_BY_CODE, 'form');

  protected onName(name: string): void {
    this.name.set(name);
    this.errors.recheck('name', this.checks()[0][1]);
  }

  protected toggleScope(scope: AccessTokenScope, checked: boolean): void {
    this.scopes.update((scopes) => {
      const next = new Set(scopes);
      if (checked) next.add(scope);
      else next.delete(scope);
      return next;
    });
    this.errors.recheck('scopes', this.checks()[1][1]);
  }

  protected async submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (this.pending()) return;
    if (!this.errors.check(this.checks())) {
      this.summary().focusFirstError();
      return;
    }
    this.pending.set(true);
    try {
      this.created.emit(await this.api.create(this.request()));
    } catch (error) {
      this.errors.set(error);
      this.summary().focusFirstError();
    } finally {
      this.pending.set(false);
    }
  }

  private checks(): FieldCheck<CreationField>[] {
    const max = TOKEN_LIMITS.nameMaxChars;
    return [
      ['name', fieldError(isValidName(this.name()), 'errors.token.name', { max })],
      ['scopes', fieldError(this.scopes().size > 0, 'tokens.create.scopes_required')],
    ];
  }

  private request(): { name: string; scopes: AccessTokenScope[]; expiresInDays: number } {
    const scopes = SCOPES.filter((scope) => this.scopes().has(scope));
    return { name: this.name().trim(), scopes, expiresInDays: this.expiryDays() };
  }
}
