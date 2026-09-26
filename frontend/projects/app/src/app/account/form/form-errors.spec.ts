import { ApiProblem } from 'shared';
import { FormErrors } from './form-errors';

type Field = 'email' | 'password' | 'form';

const REQUIRED = { key: 'auth.form.email_required', params: {} };
const TOO_SHORT = { key: 'errors.auth.password_length', params: { min: 12, max: 128 } };

function setUp(): FormErrors<Field> {
  return new FormErrors<Field>({ 'auth.email_invalid': 'email' }, 'form');
}

describe('FormErrors', () => {
  it('records a mapped code against its field, as its errors.<code> message', () => {
    const errors = setUp();

    const field = errors.set(new ApiProblem(422, 'auth.email_invalid', { max: 254 }));

    expect(field).toBe('email');
    expect(errors.of('email')).toEqual({ key: 'errors.auth.email_invalid', params: { max: 254 } });
    expect(errors.of('form')).toBeUndefined();
  });

  it('sends an unmapped code to the form field', () => {
    const errors = setUp();

    const field = errors.set(new ApiProblem(403, 'auth.csrf'));

    expect(field).toBe('form');
    expect(errors.of('form')?.key).toBe('errors.auth.csrf');
  });

  it('replaces every error on a new submission', () => {
    const errors = setUp();
    errors.check([
      ['email', REQUIRED],
      ['password', TOO_SHORT],
    ]);

    errors.set(new ApiProblem(403, 'auth.csrf'));

    expect(errors.entries().map((entry) => entry.field)).toEqual(['form']);
  });

  it('keeps the errors the checks found in the order of the fields', () => {
    const errors = setUp();

    const valid = errors.check([
      ['email', REQUIRED],
      ['password', TOO_SHORT],
    ]);

    expect(valid).toBe(false);
    expect(errors.entries()).toEqual([
      { field: 'email', error: REQUIRED },
      { field: 'password', error: TOO_SHORT },
    ]);
  });

  it('lets the form be sent when no check found anything', () => {
    const errors = setUp();

    expect(
      errors.check([
        ['email', undefined],
        ['password', undefined],
      ]),
    ).toBe(true);
    expect(errors.entries()).toEqual([]);
  });

  it('checks a field again once it changed, only if it was invalid', () => {
    const errors = setUp();
    errors.check([['email', REQUIRED]]);

    errors.recheck('password', TOO_SHORT);
    errors.recheck('email', undefined);

    expect(errors.entries()).toEqual([]);
  });

  it('clears every error', () => {
    const errors = setUp();
    errors.set(new ApiProblem(422, 'auth.email_invalid'));

    errors.clear();

    expect(errors.entries()).toEqual([]);
  });

  it('rethrows anything that is not an ApiProblem', () => {
    const errors = setUp();
    const bug = new Error('boom');

    expect(() => errors.set(bug)).toThrow(bug);
  });
});
