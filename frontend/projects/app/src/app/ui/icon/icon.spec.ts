import { TestBed, type ComponentFixture } from '@angular/core/testing';
import { Icon } from './icon';
import { ICON_PATHS, type IconName, type IconShape } from './icon-paths';

// SVG path data: commands, numbers, separators; anything else is a typo the browser drops.
const PATH_DATA = /^[MmLlHhVvCcSsQqTtAaZz\d.,\s-]+$/;

function render(name: IconName, size?: 'small' | 'large'): ComponentFixture<Icon> {
  const fixture = TestBed.createComponent(Icon);
  fixture.componentRef.setInput('name', name);
  if (size) fixture.componentRef.setInput('size', size);
  fixture.detectChanges();
  return fixture;
}

function paths(fixture: ComponentFixture<Icon>): string[] {
  const elements = fixture.nativeElement.querySelectorAll('path') as NodeListOf<SVGPathElement>;
  return Array.from(elements, (path) => path.getAttribute('d') ?? '');
}

describe('Icon', () => {
  it('is hidden from assistive technology and never takes the focus', () => {
    const fixture = render('pencil');
    const host = fixture.nativeElement as HTMLElement;

    expect(host.getAttribute('aria-hidden')).toBe('true');
    expect(host.querySelector('svg')?.getAttribute('focusable')).toBe('false');
  });

  it('draws the registered outline of its name', () => {
    expect(paths(render('pencil'))).toEqual([ICON_PATHS.pencil.outline]);
    expect(paths(render('trash'))).toEqual([ICON_PATHS.trash.outline]);
  });

  it('fills the solid part of an icon that has one', () => {
    const fixture = render('rectangle-filled');

    expect(paths(fixture)).toHaveLength(2);
    expect(fixture.nativeElement.querySelector('path.solid')).not.toBeNull();
  });

  it('takes the size it is given', () => {
    expect(render('plus', 'small').nativeElement.classList).toContain('lp-icon--small');
    expect(render('plus', 'large').nativeElement.classList).toContain('lp-icon--large');
  });

  it('registers only well-formed path data', () => {
    for (const shape of Object.values<IconShape>(ICON_PATHS)) {
      expect(shape.outline).toMatch(PATH_DATA);
      if (shape.solid) expect(shape.solid).toMatch(PATH_DATA);
    }
  });
});
