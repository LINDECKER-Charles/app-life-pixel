import {
  ChangeDetectionStrategy,
  Component,
  computed,
  DestroyRef,
  effect,
  inject,
  linkedSignal,
  signal,
  untracked,
} from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { EDITOR_ENGINE } from '../engine/editor-engine';
import { isEngineError, type Framework, type SnippetRequest } from '../engine/engine-types';
import { ExportFlow } from './export-flow';
import { defaultLoader, defaultSrc } from './export-snippet-values';

const FRAMEWORKS: readonly Framework[] = ['html', 'angular', 'react', 'vue'];
const COPIED_DURATION_MS = 2000;

function isFramework(value: string): value is Framework {
  return (FRAMEWORKS as readonly string[]).includes(value);
}

/**
 * The integration snippet: a framework picker, the URLs, the tag and the alternative text, the
 * code from `engine.snippet`, and a copy button (editor.md, U5).
 */
@Component({
  selector: 'lp-export-snippet',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './export-snippet.html',
  styleUrl: './export-snippet.scss',
})
export class ExportSnippet {
  private readonly engine = inject(EDITOR_ENGINE);
  private readonly flow = inject(ExportFlow);
  private readonly copiedTimeout = signal<ReturnType<typeof setTimeout> | undefined>(undefined);
  private requestId = 0;

  protected readonly frameworks = FRAMEWORKS;
  protected readonly framework = signal<Framework>('html');

  private readonly wasmRow = computed(
    () => this.flow.rows().find((row) => row.format === 'wasm') ?? null,
  );

  protected readonly src = linkedSignal(() => defaultSrc(this.wasmRow()));
  protected readonly loader = linkedSignal(() => defaultLoader(this.wasmRow()));
  protected readonly alt = linkedSignal(() => this.flow.document()?.title ?? '');
  protected readonly tag = signal('');
  protected readonly tags = computed(() => this.flow.document()?.tags ?? []);

  protected readonly code = signal('');
  protected readonly copied = signal(false);

  constructor() {
    effect(() => {
      const request: SnippetRequest = {
        framework: this.framework(),
        src: this.src(),
        loader: this.loader(),
        tag: this.tag() === '' ? undefined : this.tag(),
        alt: this.alt(),
      };
      untracked(() => void this.refresh(request));
    });
    inject(DestroyRef).onDestroy(() => clearTimeout(this.copiedTimeout()));
  }

  protected onFrameworkChange(value: string): void {
    if (isFramework(value)) this.framework.set(value);
  }

  protected async copy(): Promise<void> {
    await navigator.clipboard.writeText(this.code());
    this.copied.set(true);
    clearTimeout(this.copiedTimeout());
    this.copiedTimeout.set(setTimeout(() => this.copied.set(false), COPIED_DURATION_MS));
  }

  private async refresh(request: SnippetRequest): Promise<void> {
    const id = ++this.requestId;
    try {
      const code = await this.engine.snippet(request);
      if (id === this.requestId) this.code.set(code);
    } catch (error) {
      if (!isEngineError(error)) throw error;
      if (id === this.requestId) this.code.set('');
    }
  }
}
