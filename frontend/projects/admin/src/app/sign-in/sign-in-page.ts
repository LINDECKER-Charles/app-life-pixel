import { ChangeDetectionStrategy, Component, computed, inject, input, signal } from '@angular/core';
import { Router } from '@angular/router';
import { TranslocoPipe } from '@jsverse/transloco';
import type { ApiProblem } from 'shared';
import { asProblem, problemMessage } from '../core/problem';
import { SessionStore } from '../core/session-store';

/** Six digits: the current code of the admin's authenticator. */
const CODE_PATTERN = /^\d{6}$/;

/** A return address inside the console, never another origin. */
export function safeReturnUrl(returnUrl: string | undefined): string {
  return returnUrl?.startsWith('/') && !returnUrl.startsWith('//') ? returnUrl : '/';
}

/**
 * Signing in (support-admin.md, H11): the address, the password and the authenticator's code,
 * together. A wrong one of the three gives the same answer.
 */
@Component({
  selector: 'lp-sign-in-page',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './sign-in-page.html',
  styleUrl: './sign-in-page.scss',
})
export class SignInPage {
  private readonly session = inject(SessionStore);
  private readonly router = inject(Router);

  /** Bound from the query: where to go once signed in. */
  readonly returnUrl = input<string>();

  protected readonly pending = signal(false);
  /** The form was sent incomplete, or with a code that is not six digits. */
  protected readonly invalid = signal(false);
  protected readonly problem = signal<ApiProblem | null>(null);
  protected readonly error = computed(() => {
    const problem = this.problem();
    return problem ? problemMessage(problem) : null;
  });

  protected async submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    const form = new FormData(event.target as HTMLFormElement);
    const email = String(form.get('email') ?? '').trim();
    const password = String(form.get('password') ?? '');
    const code = String(form.get('code') ?? '').replace(/\s/g, '');
    if (!email || !password || !CODE_PATTERN.test(code)) {
      this.problem.set(null);
      this.invalid.set(true);
      return;
    }
    this.invalid.set(false);
    this.pending.set(true);
    this.problem.set(null);
    try {
      await this.session.signIn({ email, password, code });
      await this.router.navigateByUrl(safeReturnUrl(this.returnUrl()));
    } catch (error: unknown) {
      this.problem.set(asProblem(error));
    } finally {
      this.pending.set(false);
    }
  }
}
