/*
 * The editor engine's public vocabulary: documents, editing operations, rendering, export and
 * errors. Every engine implementation (the mock, and W1's WasmEditorEngine) shares these types,
 * so no front-end code depends on how the engine is built.
 */

export type LayerId = number;
export type FrameId = number;
export type Color = string; // "#rrggbbaa"
export type LoopMode = 'loop' | 'once';

export interface Point {
  readonly x: number;
  readonly y: number;
}

export interface Area {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

/** The JSON shape of core's Tag: `loop_mode` is serialized as `loop`. */
export interface TagSpec {
  readonly name: string;
  readonly first: number;
  readonly last: number;
  readonly loop: LoopMode;
}

/** Mirrors core::edit::Operation field for field (core.md, K2). */
export type EditOperation =
  | {
      readonly kind: 'paintStroke';
      readonly layer: LayerId;
      readonly frame: FrameId;
      readonly points: readonly Point[];
      readonly index: number;
    }
  | {
      readonly kind: 'fill';
      readonly layer: LayerId;
      readonly frame: FrameId;
      readonly at: Point;
      readonly index: number;
    }
  | {
      readonly kind: 'line';
      readonly layer: LayerId;
      readonly frame: FrameId;
      readonly from: Point;
      readonly to: Point;
      readonly index: number;
    }
  | {
      readonly kind: 'rectangle';
      readonly layer: LayerId;
      readonly frame: FrameId;
      readonly from: Point;
      readonly to: Point;
      readonly index: number;
      readonly filled: boolean;
    }
  | {
      readonly kind: 'moveSelection';
      readonly layer: LayerId;
      readonly frame: FrameId;
      readonly area: Area;
      readonly offset: Point;
    }
  | { readonly kind: 'setPaletteEntry'; readonly index: number; readonly color: Color }
  | { readonly kind: 'addPaletteEntry'; readonly color: Color }
  | { readonly kind: 'removePaletteEntry'; readonly index: number }
  | { readonly kind: 'movePaletteEntry'; readonly from: number; readonly to: number }
  | { readonly kind: 'addLayer'; readonly position: number; readonly name: string }
  | { readonly kind: 'deleteLayer'; readonly layer: LayerId }
  | { readonly kind: 'moveLayer'; readonly layer: LayerId; readonly position: number }
  | { readonly kind: 'renameLayer'; readonly layer: LayerId; readonly name: string }
  | { readonly kind: 'setLayerVisibility'; readonly layer: LayerId; readonly visible: boolean }
  | { readonly kind: 'addFrame'; readonly position: number; readonly durationMs: number }
  | { readonly kind: 'duplicateFrame'; readonly frame: FrameId }
  | { readonly kind: 'deleteFrame'; readonly frame: FrameId }
  | { readonly kind: 'moveFrame'; readonly frame: FrameId; readonly position: number }
  | { readonly kind: 'setFrameDuration'; readonly frame: FrameId; readonly durationMs: number }
  | { readonly kind: 'addTag'; readonly tag: TagSpec }
  | { readonly kind: 'updateTag'; readonly name: string; readonly tag: TagSpec }
  | { readonly kind: 'deleteTag'; readonly name: string }
  | { readonly kind: 'replaceTags'; readonly tags: readonly TagSpec[] }
  | { readonly kind: 'setTitle'; readonly title: string }
  | {
      readonly kind: 'importImage';
      readonly layer: LayerId;
      readonly frame: FrameId;
      readonly png: Uint8Array;
      readonly at: Point;
    }
  | {
      readonly kind: 'importSpriteSheet';
      readonly layer: LayerId;
      readonly position: number;
      readonly png: Uint8Array;
      readonly cellWidth: number;
      readonly cellHeight: number;
      readonly durationMs: number;
    };

export interface NewAnimationOptions {
  readonly title: string;
  readonly width: number;
  readonly height: number;
  readonly layerName: string;
  readonly frameDurationMs?: number;
  readonly palette?: readonly Color[];
}

export interface DocumentSummary {
  readonly title: string;
  readonly width: number;
  readonly height: number;
  readonly palette: readonly Color[];
  readonly layers: readonly {
    readonly id: LayerId;
    readonly name: string;
    readonly visible: boolean;
  }[];
  readonly frames: readonly { readonly id: FrameId; readonly durationMs: number }[];
  readonly tags: readonly TagSpec[];
}

export interface EngineState {
  readonly status: 'starting' | 'empty' | 'ready' | 'failed';
  readonly document: DocumentSummary | null;
  readonly canUndo: boolean;
  readonly canRedo: boolean;
  readonly hasUnsavedWork: boolean;
  readonly limits: Limits | null; // core's Limits::current()
}

export interface RenderRequest {
  readonly frame: FrameId;
  readonly preview?: EditOperation;
}

export interface RenderedFrame {
  readonly width: number;
  readonly height: number;
  readonly pixels: Uint8ClampedArray;
}

/** One readonly field per constant of core.md's limits, in camelCase. */
export interface Limits {
  readonly canvasMinSide: number;
  readonly canvasMaxSide: number;
  readonly maxFrames: number;
  readonly maxLayers: number;
  readonly maxTags: number;
  readonly maxCelPixels: number;
  readonly maxPaletteEntries: number;
  readonly minFrameDurationMs: number;
  readonly maxFrameDurationMs: number;
  readonly defaultFrameDurationMs: number;
  readonly nameMaxChars: number;
  readonly tagNameMaxChars: number;
  readonly maxDocumentBytes: number;
  readonly importMaxSide: number;
  readonly importMaxBytes: number;
  readonly strokeMaxPoints: number;
  readonly drawMaxOperations: number;
  readonly historyMaxSteps: number;
  readonly historyMaxBytes: number;
  readonly exportMinScale: number;
  readonly exportMaxScale: number;
  readonly exportMaxSide: number;
  readonly previewMaxSide: number;
  readonly previewMaxBytes: number;
  readonly passwordMinChars: number;
  readonly passwordMaxChars: number;
  readonly emailMaxChars: number;
  readonly supportMessageMaxChars: number;
  readonly screenshotMaxBytes: number;
  readonly screenshotMaxSide: number;
  readonly tokenNameMaxChars: number;
  readonly tokenExpiryDays: readonly number[];
  readonly maxActiveTokens: number;
  readonly pageSizeDefault: number;
  readonly pageSizeMax: number;
  readonly mcpPageSizeDefault: number;
  readonly mcpPageSizeMax: number;
}

/** The same names everywhere: Rust, TypeScript, MCP, product events. */
export type ExportFormat = 'wasm' | 'gif' | 'apng' | 'sprite_sheet' | 'png_frames';

export interface ExportRequest {
  readonly format: ExportFormat;
  readonly tag?: string;
  readonly scale?: number;
}

export interface ExportedFile {
  readonly name: string;
  readonly mediaType: string;
  readonly bytes: Uint8Array;
}

export interface ExportResult {
  readonly files: readonly ExportedFile[];
  readonly totalBytes: number;
}

export type Framework = 'html' | 'angular' | 'react' | 'vue';

export interface SnippetRequest {
  readonly framework: Framework;
  readonly src: string;
  readonly loader: string;
  readonly tag?: string;
  readonly alt: string;
}

export interface EngineError {
  readonly code: string;
  readonly params: Readonly<Record<string, unknown>>;
}

/** Narrows a rejection reason to an `EngineError`, for callers that must not swallow bugs. */
export function isEngineError(value: unknown): value is EngineError {
  return (
    typeof value === 'object' &&
    value !== null &&
    typeof (value as { code?: unknown }).code === 'string' &&
    typeof (value as { params?: unknown }).params === 'object'
  );
}
