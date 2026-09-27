import {
  ApplicationRef,
  ComponentRef,
  createComponent,
  Directive,
  DOCUMENT,
  effect,
  ElementRef,
  EnvironmentInjector,
  inject,
  input,
  OnDestroy,
  Renderer2,
  untracked,
} from '@angular/core';
import { TooltipBubble, type TooltipPlacement } from './tooltip-bubble';

/** The space between a control and its tooltip, in CSS pixels. */
const GAP_PX = 8;
/** How long a tooltip waits for the pointer to reach it before it closes, in milliseconds. */
const HIDE_DELAY_MS = 150;

let nextId = 0;

/**
 * A tooltip for a dense control (design-system/docs/components.md): shown on hover and on
 * keyboard focus, never on a pointer's focus; kept while the pointer moves onto it; closed by
 * Escape, a press, or leaving. It repeats the control's name with its shortcut, so it never holds
 * a required instruction. While shown, the control describes itself by it (`aria-describedby`),
 * set at once so that a screen reader reads it with the focus.
 */
@Directive({
  selector: '[lpTooltip]',
  host: {
    '(pointerenter)': 'show()',
    '(pointerleave)': 'scheduleHide()',
    '(pointerdown)': 'hide()',
    '(focusin)': 'onFocusIn()',
    '(focusout)': 'hide()',
    '(document:keydown.escape)': 'hide()',
  },
})
export class Tooltip implements OnDestroy {
  /** The control's name, translated. */
  readonly lpTooltip = input.required<string>();
  /** Its shortcut, as `describeShortcut` writes it, if it has one. */
  readonly lpTooltipShortcut = input('');
  readonly lpTooltipPlacement = input<TooltipPlacement>('right');

  private readonly host = inject<ElementRef<HTMLElement>>(ElementRef);
  private readonly appRef = inject(ApplicationRef);
  private readonly injector = inject(EnvironmentInjector);
  private readonly document = inject(DOCUMENT);
  private readonly renderer = inject(Renderer2);
  private readonly id = `lp-tooltip-${nextId++}`;
  private bubble: ComponentRef<TooltipBubble> | null = null;
  private hideTimer: ReturnType<typeof setTimeout> | null = null;

  constructor() {
    effect(() => {
      const text = this.lpTooltip();
      const shortcut = this.lpTooltipShortcut();
      untracked(() => this.update({ text, shortcut }));
    });
  }

  ngOnDestroy(): void {
    this.hide();
  }

  protected show(): void {
    this.cancelHide();
    if (!this.bubble) this.bubble = this.createBubble();
    this.update({ text: this.lpTooltip(), shortcut: this.lpTooltipShortcut() });
    this.place(this.bubble);
    this.renderer.setAttribute(this.host.nativeElement, 'aria-describedby', this.id);
  }

  protected hide(): void {
    this.cancelHide();
    if (!this.bubble) return;
    const element = this.bubble.location.nativeElement as HTMLElement;
    this.appRef.detachView(this.bubble.hostView);
    this.bubble.destroy();
    element.remove();
    this.bubble = null;
    this.renderer.removeAttribute(this.host.nativeElement, 'aria-describedby');
  }

  /** Leaving the control: stays open for keyboard focus, or while the pointer reaches the tip. */
  protected scheduleHide(): void {
    if (this.hasKeyboardFocus()) return;
    this.cancelHide();
    this.hideTimer = setTimeout(() => this.hide(), HIDE_DELAY_MS);
  }

  protected onFocusIn(): void {
    if (this.hasKeyboardFocus()) this.show();
  }

  private hasKeyboardFocus(): boolean {
    try {
      return this.host.nativeElement.matches(':focus-visible');
    } catch {
      return this.document.activeElement === this.host.nativeElement;
    }
  }

  private cancelHide(): void {
    if (this.hideTimer !== null) clearTimeout(this.hideTimer);
    this.hideTimer = null;
  }

  private createBubble(): ComponentRef<TooltipBubble> {
    const bubble = createComponent(TooltipBubble, { environmentInjector: this.injector });
    bubble.setInput('id', this.id);
    this.appRef.attachView(bubble.hostView);
    const element = bubble.location.nativeElement as HTMLElement;
    element.addEventListener('pointerenter', () => this.cancelHide());
    element.addEventListener('pointerleave', () => this.scheduleHide());
    this.document.body.append(element);
    return bubble;
  }

  private update(content: { readonly text: string; readonly shortcut: string }): void {
    if (!this.bubble) return;
    this.bubble.setInput('text', content.text);
    this.bubble.setInput('shortcut', content.shortcut);
    this.bubble.changeDetectorRef.detectChanges();
  }

  private place(bubble: ComponentRef<TooltipBubble>): void {
    const rect = this.host.nativeElement.getBoundingClientRect();
    const placement = this.lpTooltipPlacement();
    bubble.setInput('placement', placement);
    if (placement === 'right') {
      bubble.setInput('left', rect.right + GAP_PX);
      bubble.setInput('top', rect.top + rect.height / 2);
    } else {
      bubble.setInput('left', rect.left + rect.width / 2);
      bubble.setInput('top', rect.top - GAP_PX);
    }
    bubble.changeDetectorRef.detectChanges();
  }
}
