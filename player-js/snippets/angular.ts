// main.ts: define <life-pixel> once, before the application starts.
import '@life-pixel/player';

// The component that shows the animation.
import { ChangeDetectionStrategy, Component, CUSTOM_ELEMENTS_SCHEMA } from '@angular/core';

@Component({
  selector: 'app-pixel-animation',
  schemas: [CUSTOM_ELEMENTS_SCHEMA],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `<life-pixel src="{{src}}"{{tagAttribute}} alt="{{alt}}"></life-pixel>`,
})
export class {{className}} {}
