import { ChangeDetectionStrategy, Component, importProvidersFrom, signal } from '@angular/core';
import { type ComponentFixture, TestBed } from '@angular/core/testing';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import axe from 'axe-core';
import { firstValueFrom } from 'rxjs';
import { idScope } from 'shared';
import en from '../../../../../../../i18n/en.json';
import type { InspectorTab } from './inspector-tab';
import { InspectorTabs } from './inspector-tabs';

const SERIOUS_IMPACTS = ['serious', 'critical'];
const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };

@Component({
  imports: [InspectorTabs],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <lp-inspector-tabs [tabs]="tabs" label="Panels" [(selected)]="selected" />
    @for (tab of tabs; track tab.panelId) {
      <section
        role="tabpanel"
        [id]="tab.panelId"
        [attr.aria-labelledby]="tab.panelId + '-tab'"
        [hidden]="selected() !== tab.panelId"
      >
        <button type="button">Inside {{ tab.panelId }}</button>
      </section>
    }
  `,
})
class Host {
  private readonly id = idScope('inspector-tabs-host');

  readonly tabs: readonly InspectorTab[] = [
    { panelId: this.id('palette'), labelKey: 'editor.region.palette' },
    { panelId: this.id('layers'), labelKey: 'timeline.layers.heading' },
    { panelId: this.id('preview'), labelKey: 'editor.inspector.preview' },
  ];
  readonly selected = signal(this.tabs[0].panelId);
}

interface Tabs {
  readonly fixture: ComponentFixture<Host>;
  readonly tabs: HTMLButtonElement[];
  readonly panels: HTMLElement[];
}

async function render(): Promise<Tabs> {
  TestBed.configureTestingModule({
    providers: [importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING))],
  });
  await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
  const fixture = TestBed.createComponent(Host);
  document.body.append(fixture.nativeElement);
  await fixture.whenStable();
  const root = fixture.nativeElement as HTMLElement;
  return {
    fixture,
    tabs: Array.from(root.querySelectorAll<HTMLButtonElement>('[role="tab"]')),
    panels: Array.from(root.querySelectorAll<HTMLElement>('[role="tabpanel"]')),
  };
}

async function press(target: HTMLElement, key: string, fixture: ComponentFixture<Host>) {
  target.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true }));
  await fixture.whenStable();
}

/** What names `element` through its `aria-labelledby`. */
function nameOf(element: HTMLElement): string {
  const label = document.getElementById(element.getAttribute('aria-labelledby') ?? '');
  return label?.textContent?.trim() ?? '';
}

/** The panels shown, each by the name its tab gives it. */
function visiblePanels({ panels }: Tabs): string[] {
  return panels.filter((panel) => !panel.hidden).map(nameOf);
}

/** The selected tabs, by their text. */
function selectedTabs({ tabs }: Tabs): string[] {
  return tabs
    .filter((tab) => tab.getAttribute('aria-selected') === 'true')
    .map((tab) => tab.textContent?.trim() ?? '');
}

describe('InspectorTabs', () => {
  afterEach(() => document.body.replaceChildren());

  it('names the tab list and ties each tab to its panel', async () => {
    const tabs = await render();

    const list = (tabs.fixture.nativeElement as HTMLElement).querySelector('[role="tablist"]');
    expect(list?.getAttribute('aria-label')).toBe('Panels');
    expect(tabs.tabs.map((tab) => tab.textContent?.trim())).toEqual([
      'Palette',
      'Layers',
      'Preview',
    ]);
    for (const [index, tab] of tabs.tabs.entries()) {
      expect(tab.getAttribute('aria-controls')).toBe(tabs.panels[index].id);
      expect(tabs.panels[index].getAttribute('aria-labelledby')).toBe(tab.id);
    }
  });

  it('shows one panel, whose tab alone is selected and in the Tab sequence', async () => {
    const tabs = await render();

    expect(visiblePanels(tabs)).toEqual(['Palette']);
    expect(selectedTabs(tabs)).toEqual(['Palette']);
    expect(tabs.tabs.map((tab) => tab.tabIndex)).toEqual([0, -1, -1]);
  });

  it('shows the panel of a clicked tab', async () => {
    const tabs = await render();

    tabs.tabs[2].click();
    await tabs.fixture.whenStable();

    const host = tabs.fixture.componentInstance;
    expect(visiblePanels(tabs)).toEqual(['Preview']);
    expect(selectedTabs(tabs)).toEqual(['Preview']);
    expect(host.selected()).toBe(host.tabs[2].panelId);
  });

  it('moves with Left and Right Arrow, wrapping around, and selects the focused tab', async () => {
    const tabs = await render();
    tabs.tabs[0].focus();

    await press(tabs.tabs[0], 'ArrowRight', tabs.fixture);
    expect(document.activeElement).toBe(tabs.tabs[1]);
    expect(visiblePanels(tabs)).toEqual(['Layers']);

    await press(tabs.tabs[1], 'ArrowLeft', tabs.fixture);
    await press(tabs.tabs[0], 'ArrowLeft', tabs.fixture);
    expect(document.activeElement).toBe(tabs.tabs[2]);
    expect(visiblePanels(tabs)).toEqual(['Preview']);

    await press(tabs.tabs[2], 'ArrowRight', tabs.fixture);
    expect(document.activeElement).toBe(tabs.tabs[0]);
    expect(visiblePanels(tabs)).toEqual(['Palette']);
    expect(tabs.tabs.map((tab) => tab.tabIndex)).toEqual([0, -1, -1]);
  });

  it('moves to the last tab with End and to the first with Home', async () => {
    const tabs = await render();
    tabs.tabs[0].focus();

    await press(tabs.tabs[0], 'End', tabs.fixture);
    expect(document.activeElement).toBe(tabs.tabs[2]);
    expect(visiblePanels(tabs)).toEqual(['Preview']);

    await press(tabs.tabs[2], 'Home', tabs.fixture);
    expect(document.activeElement).toBe(tabs.tabs[0]);
    expect(visiblePanels(tabs)).toEqual(['Palette']);
  });

  it('leaves other keys to the page', async () => {
    const tabs = await render();
    const event = new KeyboardEvent('keydown', {
      key: 'ArrowDown',
      bubbles: true,
      cancelable: true,
    });

    tabs.tabs[0].dispatchEvent(event);
    await tabs.fixture.whenStable();

    expect(event.defaultPrevented).toBe(false);
    expect(visiblePanels(tabs)).toEqual(['Palette']);
  });

  it('has no serious accessibility violation', async () => {
    const tabs = await render();

    const { violations } = await axe.run(tabs.fixture.nativeElement);

    expect(
      violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? '')),
    ).toEqual([]);
  });
});
