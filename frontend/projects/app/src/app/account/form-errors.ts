import { signal } from '@angular/core';
import { ApiProblem } from 'shared';

/** Which field a server code belongs to; a code missing from it lands on the form's own field. */
export type FieldMap<Field extends string> = Readonly<Record<string, Field>>;

/**
 * The errors of one form submission, at most one per field: a new attempt replaces them all. A
 * code with no field of its own — `auth.csrf`, `rate_limit.exceeded` — lands on `formField`, shown
 * as a banner above the submit button (accounts.md, H7: "the server's codes shown next to their
 * field as `errors.<code>`").
 */
export class FormErrors<Field extends string> {
  private readonly errorsSignal = signal<Partial<Record<Field, ApiProblem>>>({});
  readonly errors = this.errorsSignal.asReadonly();

  constructor(
    private readonly fieldByCode: FieldMap<Field>,
    private readonly formField: Field,
  ) {}

  clear(): void {
    this.errorsSignal.set({});
  }

  /** Records `error` against its field and returns it; rethrows anything but an `ApiProblem`. */
  set(error: unknown): Field {
    if (!(error instanceof ApiProblem)) {
      throw error;
    }
    const field = this.fieldByCode[error.code] ?? this.formField;
    this.errorsSignal.set({ [field]: error } as Partial<Record<Field, ApiProblem>>);
    return field;
  }

  of(field: Field): ApiProblem | undefined {
    return this.errorsSignal()[field];
  }
}
