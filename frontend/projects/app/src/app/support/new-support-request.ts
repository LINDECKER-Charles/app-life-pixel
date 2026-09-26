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
import { TranslocoPipe } from '@jsverse/transloco';
import { type SupportCategory, SupportApi, type SupportRequestThread } from 'shared';
import { ErrorSummary } from '../account/form/error-summary';
import { fieldError } from '../account/form/field-error';
import { type FieldCheck, type FieldMap, FormErrors } from '../account/form/form-errors';
import { Icon } from '../ui/icon/icon';
import { StatusBanner } from '../ui/status-banner/status-banner';
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
 * without asking, and said so. The category and the message are checked when sent.
 */
@Component({
  selector: 'lp-new-support-request',
  imports: [ErrorSummary, Icon, StatusBanner, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './new-support-request.html',
  styleUrl: './support.scss',
})
export class NewSupportRequest {
  private readonly api = inject(SupportApi);
  private readonly screen = inject(SupportScreen);
  private readonly fileInput = viewChild<ElementRef<HTMLInputElement>>('fileInput');
  private readonly summary = viewChild.required(ErrorSummary);

  /** A request the server took. */
  readonly sent = output<SupportRequestThread>();

  protected readonly categories = Object.entries(CATEGORY_LABELS);
  protected readonly maxChars = SUPPORT_LIMITS.messageMaxChars;
  protected readonly maxMegabytes = SCREENSHOT_MAX_MEGABYTES;
  protected readonly accept = SCREENSHOT_TYPES.join(',');
  protected readonly targets = {
    category: 'support-category',
    message: 'support-message',
    screenshot: 'support-screenshot',
  };

  protected readonly category = signal<SupportCategory | undefined>(undefined);
  protected readonly message = signal('');
  protected readonly screenshot = signal<File | undefined>(undefined);
  protected readonly screenshotRefusal = signal<string | undefined>(undefined);
  protected readonly pending = signal(false);
  protected readonly confirmed = signal(false);
  protected readonly errors = new FormErrors<RequestField>(REQUEST_FIELD_BY_CODE, 'form');

  protected readonly length = computed(() => messageLength(this.message()));
  protected readonly screenshotError = computed(
    () => this.screenshotRefusal() !== undefined || this.errors.of('screenshot') !== undefined,
  );

  protected onCategory(value: string): void {
    this.category.set(value === '' ? undefined : (value as SupportCategory));
    this.errors.recheck('category', this.checks()[0][1]);
  }

  protected onMessage(event: Event): void {
    this.message.set((event.target as HTMLTextAreaElement).value);
    this.errors.recheck('message', this.checks()[1][1]);
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
    this.errors.recheck('screenshot', undefined);
  }

  protected async submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    const category = this.category();
    if (this.pending()) {
      return;
    }
    this.confirmed.set(false);
    if (!this.errors.check(this.checks()) || category === undefined) {
      this.summary().focusFirstError();
      return;
    }
    this.pending.set(true);
    try {
      const request = { category, message: this.message(), context: this.screen.context() };
      this.sent.emit(await this.api.create({ ...request, screenshot: this.screenshot() }));
      this.reset();
    } catch (error) {
      this.errors.set(error);
      this.summary().focusFirstError();
    } finally {
      this.pending.set(false);
    }
  }

  private checks(): FieldCheck<RequestField>[] {
    const length = this.length();
    const max = this.maxChars;
    return [
      ['category', fieldError(this.category() !== undefined, 'errors.support.category')],
      [
        'message',
        fieldError(length > 0 && length <= max, 'errors.support.message_length', { min: 1, max }),
      ],
    ];
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
