import { ChangeDetectionStrategy, Component, computed, inject, linkedSignal } from '@angular/core';
import { IonModal } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { EngineStore } from '../../engine/engine-store';
import type { LoopMode, TagSpec } from '../../engine/engine-types';
import { TagDialogState } from './tag-dialog-state';
import { initialValues, isValidRange, isValidTagName } from './tag-values';

/**
 * Adds a tag over a frame range, or renames, changes and deletes an existing one; a name the
 * engine refuses — invalid, or used twice — leaves the dialog open with its error notified
 * (editor.md, U3).
 */
@Component({
  selector: 'lp-tag-dialog',
  imports: [IonModal, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './tag-dialog.html',
  styleUrl: './tag-dialog.scss',
})
export class TagDialog {
  private readonly engine = inject(EngineStore);
  protected readonly state = inject(TagDialogState);

  protected readonly target = this.state.current;
  protected readonly isOpen = computed(() => this.target() !== null);
  protected readonly isEdit = computed(() => this.target()?.kind === 'edit');
  protected readonly titleKey = computed(() =>
    this.isEdit() ? 'timeline.tags.dialog.edit_title' : 'timeline.tags.dialog.add_title',
  );

  private readonly initial = computed(() => {
    const target = this.target();
    return target ? initialValues(target) : null;
  });

  protected readonly limits = this.engine.limits;
  protected readonly frameCount = computed(() => this.engine.document()?.frames.length ?? 0);

  protected readonly name = linkedSignal(() => this.initial()?.name ?? '');
  protected readonly first = linkedSignal(() => this.initial()?.first ?? 0);
  protected readonly last = linkedSignal(() => this.initial()?.last ?? 0);
  protected readonly loop = linkedSignal<LoopMode>(() => this.initial()?.loop ?? 'loop');

  protected readonly isNameValid = computed(() => {
    const limits = this.limits();
    return limits !== null && isValidTagName(this.name(), limits);
  });
  protected readonly isRangeValid = computed(() =>
    isValidRange(this.first(), this.last(), this.frameCount()),
  );
  protected readonly isValid = computed(() => this.isNameValid() && this.isRangeValid());

  protected async submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    const target = this.target();
    if (!target || !this.isValid()) return;
    const tag: TagSpec = {
      name: this.name().trim(),
      first: this.first(),
      last: this.last(),
      loop: this.loop(),
    };
    if (target.kind === 'add') await this.engine.apply({ kind: 'addTag', tag });
    else await this.engine.apply({ kind: 'updateTag', name: target.tag.name, tag });
    if (this.wasApplied(tag)) this.state.close();
  }

  protected async delete(): Promise<void> {
    const target = this.target();
    if (!target || target.kind !== 'edit') return;
    await this.engine.apply({ kind: 'deleteTag', name: target.tag.name });
    this.state.close();
  }

  protected close(): void {
    this.state.close();
  }

  private wasApplied(tag: TagSpec): boolean {
    return (this.engine.document()?.tags ?? []).some(
      (candidate) =>
        candidate.name === tag.name &&
        candidate.first === tag.first &&
        candidate.last === tag.last &&
        candidate.loop === tag.loop,
    );
  }
}
