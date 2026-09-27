import { ChangeDetectionStrategy, Component, importProvidersFrom, signal } from '@angular/core';
import { type ComponentFixture, TestBed } from '@angular/core/testing';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import axe from 'axe-core';
import { firstValueFrom } from 'rxjs';
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
  readonly tabs: readonly InspectorTab[] = [
    { panelId: 'palette', labelKey: 'editor.region.palette' },
    { panelId: 'layers', labelKey: 'timeline.layers.heading' },
    { panelId: 'preview', labelKey: 'editor.inspector.preview' },
  ];
  readonly selected = signal('palette');
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

function visiblePanels({ panels }: Tabs): string[] {
  return panels.filter((panel) => !panel.hidden).map((panel) => panel.id);
}

function selectedTabs({ tabs }: Tabs): string[] {
  return tabs.filter((tab) => tab.getAttribute('aria-selected') === 'true').map((tab) => tab.id);
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

    expect(visiblePanels(tabs)).toEqual(['palette']);
    expect(selectedTabs(tabs)).toEqual(['palette-tab']);
    expect(tabs.tabs.map((tab) => tab.tabIndex)).toEqual([0, -1, -1]);
  });

  it('shows the panel of a clicked tab', async () => {
    const tabs = await render();

    tabs.tabs[2].click();
    await tabs.fixture.whenStable();

    expect(visiblePanels(tabs)).toEqual(['preview']);
    expect(selectedTabs(tabs)).toEqual(['preview-tab']);
    expect(tabs.fixture.componentInstance.selected()).toBe('preview');
  });

  it('moves with Left and Right Arrow, wrapping around, and selects the focused tab', async () => {
    const tabs = await render();
    tabs.tabs[0].focus();

    await press(tabs.tabs[0], 'ArrowRight', tabs.fixture);
    expect(document.activeElement).toBe(tabs.tabs[1]);
    expect(visiblePanels(tabs)).toEqual(['layers']);

    await press(tabs.tabs[1], 'ArrowLeft', tabs.fixture);
    await press(tabs.tabs[0], 'ArrowLeft', tabs.fixture);
    expect(document.activeElement).toBe(tabs.tabs[2]);
    expect(visiblePanels(tabs)).toEqual(['preview']);

    await press(tabs.tabs[2], 'ArrowRight', tabs.fixture);
    expect(document.activeElement).toBe(tabs.tabs[0]);
    expect(visiblePanels(tabs)).toEqual(['palette']);
    expect(tabs.tabs.map((tab) => tab.tabIndex)).toEqual([0, -1, -1]);
  });

  it('moves to the last tab with End and to the first with Home', async () => {
    const tabs = await render();
    tabs.tabs[0].focus();

    await press(tabs.tabs[0], 'End', tabs.fixture);
    expect(document.activeElement).toBe(tabs.tabs[2]);
    expect(visiblePanels(tabs)).toEqual(['preview']);

    await press(tabs.tabs[2], 'Home', tabs.fixture);
    expect(document.activeElement).toBe(tabs.tabs[0]);
    expect(visiblePanels(tabs)).toEqual(['palette']);
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
    expect(visiblePanels(tabs)).toEqual(['palette']);
  });

  it('has no serious accessibility violation', async () => {
    const tabs = await render();

    const { violations } = await axe.run(tabs.fixture.nativeElement);

    expect(
      violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? '')),
    ).toEqual([]);
  });
});
