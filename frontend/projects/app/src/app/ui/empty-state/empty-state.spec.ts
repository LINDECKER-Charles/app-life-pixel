import { ChangeDetectionStrategy, Component, signal } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import axe from 'axe-core';
import { EmptyState } from './empty-state';

const SERIOUS_IMPACTS = ['serious', 'critical'];

@Component({
  imports: [EmptyState],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <lp-empty-state
      heading="No saved projects yet"
      description="Create an animation, then save it here."
      [hasPip]="hasPip()"
      [headingLevel]="level()"
    >
      <button type="button" class="lp-button lp-button--primary" (click)="created = true">
        Create animation
      </button>
    </lp-empty-state>
  `,
})
class Host {
  readonly hasPip = signal(false);
  readonly level = signal<2 | 3>(2);
  created = false;
}

function render(hasPip = false): HTMLElement {
  const fixture = TestBed.createComponent(Host);
  fixture.componentInstance.hasPip.set(hasPip);
  fixture.detectChanges();
  return fixture.nativeElement;
}

describe('EmptyState', () => {
  it('titles the region and explains it', () => {
    const root = render();

    expect(root.querySelector('h2')?.textContent).toBe('No saved projects yet');
    expect(root.querySelector('p')?.textContent).toBe('Create an animation, then save it here.');
  });

  it('puts the title at the level the page needs', () => {
    const fixture = TestBed.createComponent(Host);
    fixture.componentInstance.level.set(3);
    fixture.detectChanges();

    expect(fixture.nativeElement.querySelector('h3')?.textContent).toBe('No saved projects yet');
    expect(fixture.nativeElement.querySelector('h2')).toBeNull();
  });

  it('offers the projected action', () => {
    const fixture = TestBed.createComponent(Host);
    fixture.detectChanges();

    const action: HTMLButtonElement = fixture.nativeElement.querySelector('.actions button');
    action.click();

    expect(action.textContent?.trim()).toBe('Create animation');
    expect(fixture.componentInstance.created).toBe(true);
  });

  it('shows Pip only when asked, as a decorative image', () => {
    expect(render().querySelector('img')).toBeNull();

    const pip = render(true).querySelector('img');
    expect(pip?.getAttribute('alt')).toBe('');
    expect(pip?.getAttribute('src')).toBe('/design-system/pip.svg');
  });

  it('has no serious accessibility violation', async () => {
    const { violations } = await axe.run(render(true));

    expect(
      violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? '')),
    ).toEqual([]);
  });
});
