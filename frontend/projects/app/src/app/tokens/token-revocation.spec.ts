import { TestBed } from '@angular/core/testing';
import { AlertController } from '@ionic/angular';
import { TokensApi } from 'shared';
import { ACCOUNT, openAccountPage } from '../account/testing/account-test-support';
import { TokensPage } from './tokens-page';
import { TokenRevocation } from './token-revocation';

/** Answers the next alert by `role`, and returns the options it was created with. */
function answerAlert(role: string): ReturnType<typeof vi.fn> {
  const alerts = TestBed.inject(AlertController);
  const alert = {
    present: vi.fn().mockResolvedValue(undefined),
    onDidDismiss: vi.fn().mockResolvedValue({ role }),
  };
  return vi.spyOn(alerts, 'create').mockResolvedValue(alert as never) as never;
}

describe('TokenRevocation', () => {
  it('asks by the token name, and confirms on the destructive choice only', async () => {
    const api = { list: vi.fn().mockResolvedValue([]) };
    TestBed.configureTestingModule({ providers: [{ provide: TokensApi, useValue: api }] });
    await openAccountPage(TokensPage, ACCOUNT);
    const revocation = TestBed.inject(TokenRevocation);

    const create = answerAlert('destructive');
    await expect(revocation.confirm('Laptop')).resolves.toBe(true);
    const options = create.mock.calls[0][0];
    expect(options.header).toBe('Revoke this token?');
    expect(options.message).toContain('“Laptop”');
    create.mockRestore();

    answerAlert('cancel');
    await expect(revocation.confirm('Laptop')).resolves.toBe(false);
  });
});
