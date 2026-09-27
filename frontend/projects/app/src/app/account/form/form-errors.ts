import { computed, signal } from '@angular/core';
import { ApiProblem, type ProblemParams } from 'shared';

/** Which field a server code belongs to; a code missing from it lands on the form's own field. */
export type FieldMap<Field extends string> = Readonly<Record<string, Field>>;

/** A message to show against a field: its i18n key and the key's parameters. */
export interface FormError {
  readonly key: string;
  readonly params: ProblemParams;
}

/** One error of the form, for the summary: the field it belongs to and its message. */
export interface FormErrorEntry {
  readonly field: string;
  readonly error: FormError;
}

/** What a field's check found: its message, or `undefined` when the value is fine. */
export type FieldCheck<Field extends string> = readonly [Field, FormError | undefined];

/**
 * The errors of one form submission, in the order of the form's fields: a new attempt replaces
 * them all. A server code with no field of its own — `auth.csrf`, `rate_limit.exceeded` — lands on
 * `formField`, shown in the error summary only (accounts.md, H7: "the server's codes shown next to
 * their field as `errors.<code>`"). The form's own checks run on submit, and again on a field that
 * was invalid once its value changes (design-system/docs/patterns.md, "Forms and validation").
 */
export class FormErrors<Field extends string> {
  private readonly errorsSignal = signal<ReadonlyMap<Field, FormError>>(new Map());

  /** Every error, in the order of the fields. */
  readonly entries = computed<readonly FormErrorEntry[]>(() =>
    [...this.errorsSignal()].map(([field, error]) => ({ field, error })),
  );

  constructor(
    private readonly fieldByCode: FieldMap<Field>,
    private readonly formField: Field,
  ) {}

  clear(): void {
    this.errorsSignal.set(new Map());
  }

  /** Records a server `error` against its field and returns it; rethrows anything else. */
  set(error: unknown): Field {
    if (!(error instanceof ApiProblem)) {
      throw error;
    }
    const field = this.fieldByCode[error.code] ?? this.formField;
    const message = { key: `errors.${error.code}`, params: error.params };
    this.errorsSignal.set(new Map([[field, message]]));
    return field;
  }

  /**
   * Replaces the errors with what `checks` found, given in the order of the fields; returns
   * whether the form may be sent.
   */
  check(checks: readonly FieldCheck<Field>[]): boolean {
    const found = checks.filter((check): check is [Field, FormError] => check[1] !== undefined);
    this.errorsSignal.set(new Map(found));
    return found.length === 0;
  }

  /** Checks `field` again once its value changed, if it had an error: never a new error early. */
  recheck(field: Field, error: FormError | undefined): void {
    const errors = this.errorsSignal();
    if (!errors.has(field)) {
      return;
    }
    const next = new Map(errors);
    if (error) {
      next.set(field, error);
    } else {
      next.delete(field);
    }
    this.errorsSignal.set(next);
  }

  of(field: Field): FormError | undefined {
    return this.errorsSignal().get(field);
  }
}
