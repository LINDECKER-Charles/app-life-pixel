import {
  ChangeDetectionStrategy,
  Component,
  computed,
  ElementRef,
  inject,
  output,
  signal,
  viewChild,
} from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { type AccessTokenScope, type CreatedAccessToken, TokensApi } from 'shared';
import { type FieldMap, FormErrors } from '../account/form-errors';
import {
  DEFAULT_SCOPES,
  isValidName,
  SCOPE_HINTS,
  SCOPE_LABELS,
  SCOPES,
  TOKEN_LIMITS,
} from './token-values';

type CreationField = 'name' | 'expiry' | 'form';

const FIELD_BY_CODE: FieldMap<CreationField> = {
  'token.name': 'name',
  'token.expiry': 'expiry',
};

/**
 * A new token's name, scopes — read and write unless changed — and lifetime, 90 days unless
 * changed (mcp-cli.md, A3). Emits the token the server created, its secret in it.
 */
@Component({
  selector: 'lp-token-creation-form',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './token-creation-form.html',
  styleUrl: './token-dialog.scss',
})
export class TokenCreationForm {
  private readonly api = inject(TokensApi);
  private readonly banner = viewChild<ElementRef<HTMLElement>>('banner');
  private readonly nameField = viewChild<ElementRef<HTMLInputElement>>('nameField');

  /** The token just created. */
  readonly created = output<CreatedAccessToken>();
  /** The person gave up. */
  readonly cancelled = output();

  protected readonly limits = TOKEN_LIMITS;
  protected readonly scopeOptions = SCOPES;
  protected readonly scopeLabels = SCOPE_LABELS;
  protected readonly scopeHints = SCOPE_HINTS;

  protected readonly name = signal('');
  protected readonly scopes = signal<ReadonlySet<AccessTokenScope>>(new Set(DEFAULT_SCOPES));
  protected readonly expiryDays = signal<number>(TOKEN_LIMITS.defaultExpiryDays);
  protected readonly pending = signal(false);
  protected readonly errors = new FormErrors<CreationField>(FIELD_BY_CODE, 'form');

  protected readonly isNameValid = computed(() => isValidName(this.name()));
  protected readonly isValid = computed(() => this.isNameValid() && this.scopes().size > 0);

  protected toggleScope(scope: AccessTokenScope, checked: boolean): void {
    this.scopes.update((scopes) => {
      const next = new Set(scopes);
      if (checked) next.add(scope);
      else next.delete(scope);
      return next;
    });
  }

  protected async submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (this.pending() || !this.isValid()) return;
    this.errors.clear();
    this.pending.set(true);
    try {
      this.created.emit(await this.api.create(this.request()));
    } catch (error) {
      const field = this.errors.set(error);
      const target = field === 'name' ? this.nameField() : this.banner();
      target?.nativeElement.focus();
    } finally {
      this.pending.set(false);
    }
  }

  private request(): { name: string; scopes: AccessTokenScope[]; expiresInDays: number } {
    const scopes = SCOPES.filter((scope) => this.scopes().has(scope));
    return { name: this.name().trim(), scopes, expiresInDays: this.expiryDays() };
  }
}
