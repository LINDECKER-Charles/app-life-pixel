import { ComponentFixture, TestBed } from '@angular/core/testing';
import { Router } from '@angular/router';
import { ApiProblem } from 'shared';
import { vi } from 'vitest';
import { SessionStore } from '../core/session-store';
import { openPage, seriousViolations, settled, textOf } from '../testing/admin-test-support';
import { safeReturnUrl, SignInPage } from './sign-in-page';

async function openSignIn(
  signIn: ReturnType<typeof vi.fn>,
  returnUrl?: string,
): Promise<ComponentFixture<SignInPage>> {
  return openPage(SignInPage, {
    url: '/sign-in',
    session: null,
    inputs: returnUrl ? { returnUrl } : {},
    providers: [{ provide: SessionStore, useValue: { signIn } }],
  });
}

function fill(fixture: ComponentFixture<SignInPage>, values: Record<string, string>): void {
  for (const [name, value] of Object.entries(values)) {
    (fixture.nativeElement.querySelector(`input[name="${name}"]`) as HTMLInputElement).value =
      value;
  }
  fixture.nativeElement.querySelector('form').dispatchEvent(new Event('submit'));
}

describe('SignInPage', () => {
  it('signs in with the address, the password and the code, then goes back', async () => {
    const signIn = vi.fn().mockResolvedValue(undefined);
    const fixture = await openSignIn(signIn, '/users?env=staging');

    fill(fixture, { email: ' ops@example.com ', password: 'secret', code: '123 456' });

    expect(signIn).toHaveBeenCalledWith({
      email: 'ops@example.com',
      password: 'secret',
      code: '123456',
    });
    await settled(() => expect(TestBed.inject(Router).url).toBe('/users?env=staging'));
  });

  it('asks for all three, and six digits of code', async () => {
    const signIn = vi.fn();
    const fixture = await openSignIn(signIn);

    fill(fixture, { email: 'ops@example.com', password: 'secret', code: '12345' });
    await fixture.whenStable();

    expect(signIn).not.toHaveBeenCalled();
    expect(textOf(fixture.nativeElement.querySelector('[role="alert"]'))).toBe(
      'Enter your address, your password and the 6 digits of your code.',
    );
  });

  it('says a wrong address, password or code the same way', async () => {
    const signIn = vi.fn().mockRejectedValue(new ApiProblem(401, 'admin.invalid_credentials'));
    const fixture = await openSignIn(signIn);

    fill(fixture, { email: 'ops@example.com', password: 'wrong', code: '000000' });

    await settled(() =>
      expect(textOf(fixture.nativeElement.querySelector('[role="alert"]'))).toBe(
        'The email address, the password or the code is wrong.',
      ),
    );
  });

  it('returns only inside the console', () => {
    expect(safeReturnUrl('/logs?env=staging')).toBe('/logs?env=staging');
    expect(safeReturnUrl('//evil.example')).toBe('/');
    expect(safeReturnUrl('https://evil.example')).toBe('/');
    expect(safeReturnUrl(undefined)).toBe('/');
  });

  it('passes axe', async () => {
    const fixture = await openSignIn(vi.fn());

    expect(await seriousViolations(fixture.nativeElement)).toEqual([]);
  });
});
