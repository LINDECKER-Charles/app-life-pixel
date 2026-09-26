import { ApiProblem } from 'shared';
import { FormErrors } from './form-errors';

type Field = 'email' | 'password' | 'form';

function setUp(): FormErrors<Field> {
  return new FormErrors<Field>({ 'auth.email_invalid': 'email' }, 'form');
}

describe('FormErrors', () => {
  it('records a mapped code against its field', () => {
    const errors = setUp();
    const problem = new ApiProblem(422, 'auth.email_invalid');

    const field = errors.set(problem);

    expect(field).toBe('email');
    expect(errors.of('email')).toBe(problem);
    expect(errors.of('form')).toBeUndefined();
  });

  it('sends an unmapped code to the form field', () => {
    const errors = setUp();
    const problem = new ApiProblem(403, 'auth.csrf');

    const field = errors.set(problem);

    expect(field).toBe('form');
    expect(errors.of('form')).toBe(problem);
  });

  it('replaces every error on a new submission', () => {
    const errors = setUp();
    errors.set(new ApiProblem(422, 'auth.email_invalid'));

    errors.set(new ApiProblem(403, 'auth.csrf'));

    expect(errors.of('email')).toBeUndefined();
    expect(errors.of('form')).toBeDefined();
  });

  it('clears every error', () => {
    const errors = setUp();
    errors.set(new ApiProblem(422, 'auth.email_invalid'));

    errors.clear();

    expect(errors.errors()).toEqual({});
  });

  it('rethrows anything that is not an ApiProblem', () => {
    const errors = setUp();
    const bug = new Error('boom');

    expect(() => errors.set(bug)).toThrow(bug);
  });
});
