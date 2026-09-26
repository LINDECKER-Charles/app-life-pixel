import { importProvidersFrom } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { provideIonicAngular } from '@ionic/angular';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { provideTranslocoMessageformat } from '@jsverse/transloco-messageformat';
import axe from 'axe-core';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../i18n/en.json';
import { routes } from '../app.routes';
import { desktopOnly } from '../platform/platform-guards';
import { PlatformService, type PlatformInfo } from '../platform/platform';
import { AgentsPage } from './agents-page';
import { Clipboard } from './clipboard';

const SERIOUS_IMPACTS = ['serious', 'critical'];
const INFO: PlatformInfo = {
  version: '0.1.0',
  os: 'macos',
  libraryPath: '/Users/pixel/Documents/Life Pixel',
  cliPath: '/Applications/Life Pixel.app/Contents/MacOS/life-pixel',
};
const COMMAND =
  'claude mcp add life-pixel -- "/Applications/Life Pixel.app/Contents/MacOS/life-pixel" ' +
  'mcp --library "/Users/pixel/Documents/Life Pixel"';

async function render(info: Promise<PlatformInfo>, copy = vi.fn().mockResolvedValue(undefined)) {
  TestBed.configureTestingModule({
    providers: [
      provideIonicAngular({ animated: false }),
      importProvidersFrom(
        TranslocoTestingModule.forRoot({
          langs: { en },
          translocoConfig: { availableLangs: ['en'] },
        }),
      ),
      provideTranslocoMessageformat(),
      { provide: PlatformService, useValue: { info: () => info } },
      { provide: Clipboard, useValue: { copy } },
    ],
  });
  await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
  const fixture = TestBed.createComponent(AgentsPage);
  await fixture.whenStable();
  return { root: fixture.nativeElement as HTMLElement, fixture, copy };
}

function button(root: HTMLElement, label: string): HTMLButtonElement {
  const found = Array.from(root.querySelectorAll('button')).find(
    (candidate) => candidate.textContent?.trim() === label,
  );
  if (!found) throw new Error(`no button "${label}"`);
  return found;
}

describe('the agents page', () => {
  it('is settings/agents, kept to the desktop (desktop.md, T3)', () => {
    const route = routes.find(({ path }) => path === 'settings/agents');

    expect(route?.canActivate).toContain(desktopOnly);
  });

  it('shows the claude mcp add command and the mcpServers entry with absolute paths', async () => {
    const { root } = await render(Promise.resolve(INFO));

    const blocks = Array.from(root.querySelectorAll('pre')).map((pre) => pre.textContent ?? '');
    expect(blocks[0]).toBe(COMMAND);
    expect(JSON.parse(blocks[1])).toEqual({
      mcpServers: {
        'life-pixel': { command: INFO.cliPath, args: ['mcp', '--library', INFO.libraryPath] },
      },
    });
    expect(root.textContent).toContain('Export them');
  });

  it('copies the command, and says so', async () => {
    const { root, fixture, copy } = await render(Promise.resolve(INFO));

    button(root, 'Copy the command').click();
    await fixture.whenStable();

    expect(copy).toHaveBeenCalledWith(COMMAND);
    expect(root.querySelector('lp-copy-block [role="status"]')?.textContent).toContain('Copied');
  });

  it('says when the system refuses the copy', async () => {
    const refused = vi.fn().mockRejectedValue(new Error('denied'));
    const { root, fixture } = await render(Promise.resolve(INFO), refused);

    button(root, 'Copy the JSON').click();
    await fixture.whenStable();

    const statuses = Array.from(root.querySelectorAll('lp-copy-block [role="status"]'));
    expect(statuses[1]?.textContent).toContain('Could not copy');
  });

  it('says when this build ships no CLI', async () => {
    const { root } = await render(Promise.resolve({ ...INFO, cliPath: null }));

    expect(root.querySelector('pre')).toBeNull();
    expect(root.querySelector('[role="alert"]')?.textContent).toContain(
      'cargo xtask build-desktop',
    );
  });

  it('has no serious accessibility violation', async () => {
    const { root } = await render(Promise.resolve(INFO));

    const { violations } = await axe.run(root);

    expect(
      violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? '')),
    ).toEqual([]);
  });
});
