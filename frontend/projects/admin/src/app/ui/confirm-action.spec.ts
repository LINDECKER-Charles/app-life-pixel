import { ComponentFixture } from '@angular/core/testing';
import { ApiProblem } from 'shared';
import { vi } from 'vitest';
import { openPage, seriousViolations, textOf } from '../testing/admin-test-support';
import { confirmationOf } from '../users/user-actions';
import { ConfirmAction, ConfirmRequest } from './confirm-action';

async function openConfirmation(request: ConfirmRequest): Promise<ComponentFixture<ConfirmAction>> {
  return openPage(ConfirmAction, { inputs: { request } });
}

function confirmButton(fixture: ComponentFixture<ConfirmAction>): HTMLButtonElement {
  return fixture.nativeElement.querySelector('button[type="submit"]');
}

async function typeReason(fixture: ComponentFixture<ConfirmAction>, text: string): Promise<void> {
  const reason = fixture.nativeElement.querySelector('textarea') as HTMLTextAreaElement;
  reason.value = text;
  reason.dispatchEvent(new Event('input'));
  await fixture.whenStable();
}

describe('ConfirmAction', () => {
  const suspend = confirmationOf('suspend', { email: 'lee@example.com' }, 'production');

  it('names the environment and the target', async () => {
    const fixture = await openConfirmation(suspend);
    const dialog = fixture.nativeElement.querySelector('dialog') as HTMLDialogElement;

    expect(textOf(dialog.querySelector('.environment'))).toBe('Production');
    expect(textOf(dialog.querySelector('.target'))).toBe('lee@example.com');
    expect(textOf(dialog.querySelector('#confirm-message'))).toBe(
      'lee@example.com will be signed out of production and unable to sign in until reactivated.',
    );
    expect(textOf(confirmButton(fixture))).toBe('Suspend in production');
    expect(dialog.open).toBe(true);
  });

  it('asks for a reason before confirming, and gives it trimmed', async () => {
    const fixture = await openConfirmation(suspend);
    const confirmed = vi.fn();
    fixture.componentInstance.confirmed.subscribe(confirmed);
    expect(confirmButton(fixture).disabled).toBe(true);

    await typeReason(fixture, '   ');
    expect(confirmButton(fixture).disabled).toBe(true);

    await typeReason(fixture, '  Spam, reported twice  ');
    expect(confirmButton(fixture).disabled).toBe(false);
    confirmButton(fixture).click();

    expect(confirmed).toHaveBeenCalledWith('Spam, reported twice');
  });

  it('confirms without a reason when none is recorded', async () => {
    const fixture = await openConfirmation(
      confirmationOf('export', { email: 'lee@example.com' }, 'staging'),
    );
    const confirmed = vi.fn();
    fixture.componentInstance.confirmed.subscribe(confirmed);

    expect(fixture.nativeElement.querySelector('textarea')).toBeNull();
    confirmButton(fixture).click();

    expect(confirmed).toHaveBeenCalledWith('');
  });

  it('cancels, and shows why a confirmed action failed', async () => {
    const fixture = await openConfirmation(suspend);
    const cancelled = vi.fn();
    fixture.componentInstance.cancelled.subscribe(cancelled);
    fixture.componentRef.setInput(
      'problem',
      new ApiProblem(400, 'admin.reason_length', {
        min: 1,
        max: 1000,
      }),
    );
    await fixture.whenStable();

    expect(textOf(fixture.nativeElement.querySelector('[role="alert"]'))).toBe(
      'A reason holds 1 to 1,000 characters.',
    );
    (fixture.nativeElement.querySelector('button[type="button"]') as HTMLButtonElement).click();
    expect(cancelled).toHaveBeenCalled();
  });

  it('passes axe', async () => {
    const fixture = await openConfirmation(suspend);

    expect(await seriousViolations(fixture.nativeElement)).toEqual([]);
  });
});
