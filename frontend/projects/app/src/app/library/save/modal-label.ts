import { Directive, ElementRef, inject, input } from '@angular/core';

/**
 * Names an inline `ion-modal`'s dialog. Ionic copies the host's `aria-label` to its dialog only
 * when it loads, before Angular binds a translated one: this sets it on each presentation, so
 * that assistive technologies announce the dialog by its title.
 */
@Directive({
  selector: 'ion-modal[lpModalLabel]',
  host: { '(ionModalWillPresent)': 'label()' },
})
export class ModalLabel {
  private readonly host = inject<ElementRef<HTMLElement>>(ElementRef);

  /** The dialog's accessible name, translated. */
  readonly lpModalLabel = input.required<string>();

  protected label(): void {
    const dialog = this.host.nativeElement.shadowRoot?.querySelector('[role="dialog"]');
    dialog?.setAttribute('aria-label', this.lpModalLabel());
  }
}
