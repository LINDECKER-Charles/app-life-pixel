import {
  ChangeDetectionStrategy,
  Component,
  computed,
  ElementRef,
  inject,
  output,
  signal,
  viewChild,
} from '@angular/core';
import { IonSelect, IonSelectOption, type SelectCustomEvent } from '@ionic/angular';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { type SupportCategory, SupportApi, type SupportRequestThread } from 'shared';
import { type FieldMap, FormErrors } from '../account/form-errors';
import {
  CATEGORY_LABELS,
  messageLength,
  SCREENSHOT_MAX_MEGABYTES,
  SCREENSHOT_TYPES,
  screenshotRefusal,
  SUPPORT_LIMITS,
} from './support-limits';
import { SupportScreen } from './support-screen';

type RequestField = 'category' | 'message' | 'screenshot' | 'form';

const REQUEST_FIELD_BY_CODE: FieldMap<RequestField> = {
  'support.category': 'category',
  'support.message_length': 'message',
  'support.screenshot': 'screenshot',
  'request.too_large': 'screenshot',
};

/**
 * "New request" (support-admin.md, H9): a category, a message with its character counter, and an
 * optional screenshot checked for its type and size before any upload. The context is attached
 * without asking, and said so.
 */
@Component({
  selector: 'lp-new-support-request',
  imports: [IonSelect, IonSelectOption, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './new-support-request.html',
  styleUrl: './support.scss',
})
export class NewSupportRequest {
  private readonly api = inject(SupportApi);
  private readonly screen = inject(SupportScreen);
  private readonly transloco = inject(TranslocoService);
  private readonly fileInput = viewChild<ElementRef<HTMLInputElement>>('fileInput');

  /** A request the server took. */
  readonly sent = output<SupportRequestThread>();

  protected readonly categories = Object.entries(CATEGORY_LABELS);
  protected readonly maxChars = SUPPORT_LIMITS.messageMaxChars;
  protected readonly maxMegabytes = SCREENSHOT_MAX_MEGABYTES;
  protected readonly accept = SCREENSHOT_TYPES.join(',');

  protected readonly category = signal<SupportCategory | undefined>(undefined);
  protected readonly message = signal('');
  protected readonly screenshot = signal<File | undefined>(undefined);
  protected readonly screenshotRefusal = signal<string | undefined>(undefined);
  protected readonly pending = signal(false);
  protected readonly confirmed = signal(false);
  protected readonly errors = new FormErrors<RequestField>(REQUEST_FIELD_BY_CODE, 'form');

  protected readonly length = computed(() => messageLength(this.message()));
  protected readonly ready = computed(
    () =>
      this.category() !== undefined &&
      this.length() > 0 &&
      this.length() <= this.maxChars &&
      !this.pending(),
  );

  protected errorText(field: RequestField): string | undefined {
    const error = this.errors.of(field);
    return error ? this.transloco.translate(`errors.${error.code}`, error.params) : undefined;
  }

  protected onCategory(event: SelectCustomEvent<SupportCategory>): void {
    this.category.set(event.detail.value);
  }

  protected onMessage(event: Event): void {
    this.message.set((event.target as HTMLTextAreaElement).value);
  }

  /** Keeps a PNG or JPEG within the limit; anything else is refused here, never uploaded. */
  protected onScreenshot(event: Event): void {
    const picker = event.target as HTMLInputElement;
    const file = picker.files?.[0];
    const refusal = file ? screenshotRefusal(file) : undefined;
    if (refusal) {
      picker.value = '';
    }
    this.screenshot.set(refusal ? undefined : file);
    this.screenshotRefusal.set(refusal);
  }

  protected async submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    const category = this.category();
    if (!this.ready() || category === undefined) {
      return;
    }
    this.errors.clear();
    this.confirmed.set(false);
    this.pending.set(true);
    try {
      const request = { category, message: this.message(), context: this.screen.context() };
      this.sent.emit(await this.api.create({ ...request, screenshot: this.screenshot() }));
      this.reset();
    } catch (error) {
      this.errors.set(error);
    } finally {
      this.pending.set(false);
    }
  }

  private reset(): void {
    this.category.set(undefined);
    this.message.set('');
    this.screenshot.set(undefined);
    const picker = this.fileInput()?.nativeElement;
    if (picker) {
      picker.value = '';
    }
    this.confirmed.set(true);
  }
}
