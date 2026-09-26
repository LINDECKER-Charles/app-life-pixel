import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { Router } from '@angular/router';
import { TranslocoPipe } from '@jsverse/transloco';
import { NewAnimationFlow } from '../../editor/new-animation/new-animation-flow';
import { Icon } from '../../ui/icon/icon';

const EDITOR_PATH = '/editor';

/**
 * "Create an animation" from the library (design-system/docs/journeys.md, §4): goes to the
 * editor and starts the new-animation form there, which asks first when work is unsaved.
 */
@Component({
  selector: 'lp-create-animation-button',
  imports: [Icon, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <button type="button" class="lp-button lp-button--primary" (click)="create()">
      <lp-icon name="plus" />
      {{ 'library.create_animation' | transloco }}
    </button>
  `,
})
export class CreateAnimationButton {
  private readonly router = inject(Router);
  private readonly flow = inject(NewAnimationFlow);

  protected async create(): Promise<void> {
    if (await this.router.navigateByUrl(EDITOR_PATH)) await this.flow.start();
  }
}
