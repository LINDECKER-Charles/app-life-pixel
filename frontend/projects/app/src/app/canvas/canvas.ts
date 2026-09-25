import {
  AfterViewInit,
  ChangeDetectionStrategy,
  Component,
  DestroyRef,
  ElementRef,
  OnDestroy,
  computed,
  effect,
  inject,
  signal,
  untracked,
  viewChild,
} from '@angular/core';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { EditorStore } from '../editor/editor-store';
import { Shortcuts } from '../editor/shortcuts';
import { EDITOR_ENGINE } from '../engine/editor-engine';
import { EngineStore } from '../engine/engine-store';
import type { Point, RenderedFrame } from '../engine/engine-types';
import {
  originFor,
  pixelFromClient,
  shouldShowGrid,
  type Size,
  type Viewport,
} from './geometry/canvas-geometry';
import { buildMoveSelectionOperation, type GestureContext } from './gesture/canvas-gesture';
import { CanvasInteraction } from './gesture/canvas-interaction';
import { CanvasViewport } from './gesture/canvas-viewport';
import { loadActiveFrame, loadOnionSkin, type FrameList } from './render/canvas-frame-loader';
import { drawScene, type OnionSkinFrame } from './render/canvas-renderer';

/** How far Shift with the arrows pans the view, in CSS pixels (editor.md, U2). */
const PAN_STEP_PX = 40;
const ARROW_DELTAS: Readonly<Record<string, Point>> = {
  ArrowLeft: { x: -1, y: 0 },
  ArrowRight: { x: 1, y: 0 },
  ArrowUp: { x: 0, y: -1 },
  ArrowDown: { x: 0, y: 1 },
};

/**
 * The editor's drawing surface (editor.md, U2): renders the active frame with its onion skin,
 * checkerboard and grid, and turns pointer and keyboard input into operations through
 * `CanvasInteraction` and `CanvasViewport`. Fills U1's stub in place, reading `EditorStore` and
 * `EngineStore` alone.
 */
@Component({
  selector: 'lp-canvas',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './canvas.html',
  styleUrl: './canvas.scss',
})
export class Canvas implements AfterViewInit, OnDestroy {
  private readonly engine = inject(EDITOR_ENGINE);
  private readonly engineStore = inject(EngineStore);
  private readonly store = inject(EditorStore);
  private readonly shortcuts = inject(Shortcuts);
  private readonly transloco = inject(TranslocoService);
  private readonly destroyRef = inject(DestroyRef);

  protected readonly hostRef = viewChild.required<ElementRef<HTMLDivElement>>('host');
  protected readonly viewRef = viewChild.required<ElementRef<HTMLCanvasElement>>('view');

  private readonly interaction = new CanvasInteraction();
  private readonly viewport = new CanvasViewport({
    getViewport: () => this.currentViewport(),
    setZoom: (zoom) => this.store.zoom.set(zoom),
    setPan: (pan) => this.store.pan.set(pan),
  });

  private resizeObserver: ResizeObserver | null = null;
  private renderScheduled = false;
  private renderGeneration = 0;
  private onionGeneration = 0;
  private spaceHeld = false;
  private spacePanned = false;

  private readonly viewSize = signal<Size>({ width: 0, height: 0 });
  private readonly frameImage = signal<RenderedFrame | null>(null);
  private readonly onionImages = signal<{
    readonly before: readonly OnionSkinFrame[];
    readonly after: readonly OnionSkinFrame[];
  }>({ before: [], after: [] });

  /** Shows the keyboard cursor only while the workspace itself holds the focus. */
  protected readonly focused = signal(false);
  protected readonly cursor = this.interaction.cursor;
  protected readonly showGrid = computed(
    () => this.store.showGrid() && shouldShowGrid(this.store.zoom()),
  );
  protected readonly liveMessage = computed(() =>
    this.transloco.translate('canvas.cursor.status', {
      x: this.cursor().x,
      y: this.cursor().y,
      index: this.store.colorIndex(),
    }),
  );
  /** The keyboard cursor's box, in CSS pixels relative to the visible canvas. */
  protected readonly cursorBox = computed(() => {
    const viewport = this.currentViewport();
    const origin = originFor(viewport);
    const cursor = this.cursor();
    return {
      left: origin.x + cursor.x * viewport.zoom,
      top: origin.y + cursor.y * viewport.zoom,
      size: viewport.zoom,
    };
  });

  private readonly contentSize = computed<Size | null>(() => {
    const document = this.engineStore.document();
    return document ? { width: document.width, height: document.height } : null;
  });

  private readonly context = computed<GestureContext | null>(() => {
    const layer = this.store.activeLayer();
    const frame = this.store.activeFrame();
    if (layer === null || frame === null) return null;
    return {
      layer,
      frame,
      colorIndex: this.store.colorIndex(),
      rectangleFilled: this.store.rectangleFilled(),
    };
  });

  constructor() {
    this.registerShortcuts();
    this.watchActiveFrame();
    this.watchOnionSkin();
    this.watchDraw();
  }

  ngAfterViewInit(): void {
    if (typeof ResizeObserver !== 'undefined') {
      this.resizeObserver = new ResizeObserver(() => this.onResize());
      this.resizeObserver.observe(this.hostRef().nativeElement);
    }
    this.onResize();
  }

  ngOnDestroy(): void {
    this.resizeObserver?.disconnect();
  }

  private onResize(): void {
    const host = this.hostRef().nativeElement;
    const view = this.viewRef().nativeElement;
    const size = { width: host.clientWidth, height: host.clientHeight };
    view.width = size.width;
    view.height = size.height;
    this.viewSize.set(size);
  }

  private currentViewport(): Viewport {
    return {
      view: this.viewSize(),
      content: this.contentSize() ?? { width: 0, height: 0 },
      zoom: this.store.zoom(),
      pan: this.store.pan(),
    };
  }

  // --- Rendering: fetches the active frame and its onion skin, then draws at most once per
  // animation frame (editor.md, U2). ---

  private watchActiveFrame(): void {
    effect(() => {
      const frame = this.store.activeFrame();
      const preview = this.interaction.preview(this.context());
      const generation = ++this.renderGeneration;
      untracked(() => {
        void loadActiveFrame(this.engine, frame, preview).then((image) => {
          if (generation === this.renderGeneration) this.frameImage.set(image);
        });
      });
    });
  }

  private watchOnionSkin(): void {
    effect(() => {
      const onionSkin = this.store.onionSkin();
      const activeFrame = this.store.activeFrame();
      const frames: FrameList = this.engineStore.document()?.frames ?? [];
      const generation = ++this.onionGeneration;
      untracked(() => {
        void loadOnionSkin(this.engine, { frames, activeFrame, onionSkin }).then((images) => {
          if (generation === this.onionGeneration) this.onionImages.set(images);
        });
      });
    });
  }

  private watchDraw(): void {
    effect(() => {
      this.frameImage();
      this.onionImages();
      this.viewSize();
      this.store.zoom();
      this.store.pan();
      this.showGrid();
      untracked(() => this.scheduleDraw());
    });
  }

  private scheduleDraw(): void {
    if (this.renderScheduled) return;
    this.renderScheduled = true;
    requestAnimationFrame(() => {
      this.renderScheduled = false;
      this.draw();
    });
  }

  private draw(): void {
    const ctx = this.viewRef().nativeElement.getContext('2d');
    if (!ctx) return;
    const viewport = this.currentViewport();
    const onion = this.onionImages();
    drawScene(ctx, {
      view: viewport.view,
      layout: { origin: originFor(viewport), zoom: viewport.zoom, content: viewport.content },
      frame: this.frameImage(),
      before: onion.before,
      after: onion.after,
      showGrid: this.showGrid(),
    });
  }

  // --- Pointer input: pencil, eraser, fill, line, rectangle, select, pan (editor.md, U2). ---

  protected onPointerDown(event: PointerEvent): void {
    if (event.button === 1 || this.spaceHeld) {
      event.preventDefault(); // the middle button otherwise opens the browser's autoscroll
      this.spacePanned = true;
      this.capturePointer(event);
      this.viewport.startPan(event.pointerId, { x: event.clientX, y: event.clientY });
      return;
    }
    if (event.button !== 0) return;
    const context = this.context();
    if (!context) return;
    this.capturePointer(event);
    const operation = this.interaction.pointerDown({
      pointerId: event.pointerId,
      tool: this.store.tool(),
      point: this.pixelFromEvent(event),
      context,
      selection: this.store.selection(),
    });
    if (operation) void this.engineStore.apply(operation);
  }

  protected onPointerMove(event: PointerEvent): void {
    if (this.viewport.isPanning) {
      this.viewport.updatePan(event.pointerId, { x: event.clientX, y: event.clientY });
      return;
    }
    this.interaction.pointerMove(event.pointerId, this.pixelFromEvent(event), event.shiftKey);
  }

  protected onPointerUp(event: PointerEvent): void {
    if (this.viewport.isPanning) {
      this.viewport.endPan(event.pointerId);
      return;
    }
    const context = this.context();
    this.applyGestureResult(context ? this.interaction.pointerUp(event.pointerId, context) : null);
  }

  protected onPointerCancel(event: PointerEvent): void {
    this.viewport.endPan(event.pointerId);
    this.interaction.cancel();
  }

  protected onWheel(event: WheelEvent): void {
    if (!(event.ctrlKey || event.metaKey)) return;
    event.preventDefault();
    const rect = this.viewRef().nativeElement.getBoundingClientRect();
    this.viewport.zoomWheel(
      { x: event.clientX - rect.left, y: event.clientY - rect.top },
      event.deltaY,
    );
  }

  private applyGestureResult(result: ReturnType<CanvasInteraction['pointerUp']>): void {
    if (!result) return;
    if (result.selection !== undefined) this.store.selection.set(result.selection);
    if (result.operation) void this.engineStore.apply(result.operation);
  }

  private capturePointer(event: PointerEvent): void {
    try {
      this.viewRef().nativeElement.setPointerCapture(event.pointerId);
    } catch {
      // Not every test environment implements pointer capture; the app itself always does.
    }
  }

  private pixelFromEvent(event: PointerEvent): Point {
    const rect = this.viewRef().nativeElement.getBoundingClientRect();
    const local = { x: event.clientX - rect.left, y: event.clientY - rect.top };
    return pixelFromClient(local, this.viewport.origin(), this.store.zoom());
  }

  private registerShortcuts(): void {
    const unregister = this.shortcuts.register([
      { key: '+', label: 'canvas.shortcut.zoom_in', action: () => this.viewport.zoomBy(1) },
      { key: '-', label: 'canvas.shortcut.zoom_out', action: () => this.viewport.zoomBy(-1) },
      { key: '0', label: 'canvas.shortcut.zoom_fit', action: () => this.viewport.zoomFit() },
    ]);
    this.destroyRef.onDestroy(unregister);
  }

  // --- Keyboard: the pixel cursor, Enter or Space, Alt with the arrows (editor.md, U2). ---

  protected onKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      this.interaction.cancel();
      return;
    }
    if (event.key === ' ') {
      this.onSpaceKeydown(event);
      return;
    }
    if (!(event.key in ARROW_DELTAS)) {
      if (event.key === 'Enter') this.activate();
      return;
    }
    event.preventDefault();
    if (event.altKey) this.moveSelectionByKeyboard(event.key);
    else if (event.shiftKey) this.viewport.panByStep(ARROW_DELTAS[event.key], PAN_STEP_PX);
    else this.moveCursor(event.key);
  }

  private onSpaceKeydown(event: KeyboardEvent): void {
    event.preventDefault();
    if (event.repeat) return;
    this.spaceHeld = true;
    this.spacePanned = false;
  }

  protected onKeyup(event: KeyboardEvent): void {
    if (event.key !== ' ') return;
    this.spaceHeld = false;
    if (!this.spacePanned) this.activate();
  }

  private moveCursor(key: string): void {
    const content = this.contentSize();
    if (content) this.interaction.moveCursor(ARROW_DELTAS[key], content);
  }

  private moveSelectionByKeyboard(key: string): void {
    const context = this.context();
    const selection = this.store.selection();
    if (!context || !selection) return;
    const delta = ARROW_DELTAS[key];
    this.store.selection.set({ ...selection, x: selection.x + delta.x, y: selection.y + delta.y });
    void this.engineStore.apply(buildMoveSelectionOperation(context, selection, delta));
  }

  private activate(): void {
    const context = this.context();
    if (!context) return;
    this.applyGestureResult(this.interaction.keyboardActivate(this.store.tool(), context));
  }
}
