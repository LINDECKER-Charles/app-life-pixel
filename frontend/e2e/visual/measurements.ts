import type { Page } from '@playwright/test';
import { openRoute, ROUTED_PAGE, settle } from './screens';
import { DEFAULT_ZOOM, SIZE, type VisualEditor } from './visual-editor';

/** How far the scroll check pushes the wheel: well beyond any page of the app. */
const WHEEL_STEPS = 12;
const WHEEL_DELTA_PX = 500;
/** The spacer that makes any page taller than the screen. */
const STRETCH_PX = 2000;

/** The room the canvas gets in the editor's layout, with the animation open. */
export interface StageMeasure {
  readonly viewport: { readonly width: number; readonly height: number };
  /** The stage: the region named "Canvas", around the drawing surface. */
  readonly stageHeight: number;
  readonly canvasWidth: number;
  readonly canvasHeight: number;
  readonly stageShareOfViewport: number;
  /** Whether the whole animation fits on the canvas at the default zoom. */
  readonly artworkFits: boolean;
}

/** Whether the wheel brings a page's last content into view. */
export interface ScrollResult {
  /** The content's bottom lies below the outlet before any scrolling. */
  readonly contentOverflows: boolean;
  readonly reachedBottom: boolean;
  /** How much content stays out of reach, below the outlet, after scrolling. */
  readonly hiddenAfterScrollPx: number;
}

/**
 * Whether a routed page scrolls down to its end: as it is, and stretched by a spacer taller than
 * any screen, which tells whether its container scrolls at all when the content fits.
 */
export interface ScrollMeasure {
  readonly path: string;
  readonly finalPath: string;
  readonly hasIonContent: boolean;
  readonly pageOverflow: string;
  readonly natural: ScrollResult;
  readonly stretched: ScrollResult;
}

export async function measureStage(
  editor: VisualEditor,
  viewport: { width: number; height: number },
): Promise<StageMeasure> {
  await editor.page.setViewportSize(viewport);
  await settle(editor.page);
  const none = { x: 0, y: 0, width: 0, height: 0 };
  const stage = (await editor.stage.boundingBox()) ?? none;
  const canvas = (await editor.canvas.boundingBox()) ?? none;
  const side = SIZE * DEFAULT_ZOOM;
  return {
    viewport,
    stageHeight: Math.round(stage.height),
    canvasWidth: Math.round(canvas.width),
    canvasHeight: Math.round(canvas.height),
    stageShareOfViewport: Number((stage.height / viewport.height).toFixed(3)),
    artworkFits: canvas.width >= side && canvas.height >= side,
  };
}

/**
 * Whether the editor scrolls sideways on a narrow screen: the page as a whole, and each element
 * that sticks out of the window — but those within the canvas stage and the timeline, which may
 * pan within their bounds (design-system/docs/foundations.md, "Responsive composition").
 */
export interface OverflowMeasure {
  readonly viewport: { readonly width: number; readonly height: number };
  readonly pageScrollWidth: number;
  readonly scrollsSideways: boolean;
  /** The elements outside the stage and the timeline that stick out, by tag and class. */
  readonly outside: readonly string[];
}

export async function measureOverflow(
  editor: VisualEditor,
  viewport: { width: number; height: number },
): Promise<OverflowMeasure> {
  await editor.page.setViewportSize(viewport);
  await settle(editor.page);
  const found = await editor.page.evaluate(() => {
    const width = document.documentElement.clientWidth;
    const bounded = [...document.querySelectorAll('section.stage, section.timeline')];
    const outside = [...document.querySelectorAll('body *')].filter((element) => {
      if (bounded.some((region) => region !== element && region.contains(element))) return false;
      const box = element.getBoundingClientRect();
      return box.width > 0 && box.height > 0 && (box.right > width + 0.5 || box.left < -0.5);
    });
    return {
      pageScrollWidth: document.documentElement.scrollWidth,
      scrollsSideways: document.documentElement.scrollWidth > width,
      outside: outside.map((element) => [element.localName, ...element.classList].join('.')),
    };
  });
  return { viewport, ...found };
}

interface Layout {
  readonly contentBottom: number;
  readonly outletBottom: number;
  readonly hasIonContent: boolean;
  readonly pageOverflow: string;
}

function readLayout(page: Page): Promise<Layout> {
  return page.evaluate((selector) => {
    const host = document.querySelector(selector);
    const outlet = document.querySelector('ion-router-outlet');
    const boxes = [...(host?.querySelectorAll('*') ?? [])]
      .map((element) => element.getBoundingClientRect())
      .filter((box) => box.width > 0 && box.height > 0);
    return {
      contentBottom: Math.max(0, ...boxes.map((box) => box.bottom)),
      outletBottom: Math.min(
        outlet?.getBoundingClientRect().bottom ?? window.innerHeight,
        window.innerHeight,
      ),
      hasIonContent: host?.querySelector('ion-content') !== null,
      pageOverflow: host ? getComputedStyle(host).overflowY : 'none',
    };
  }, ROUTED_PAGE);
}

/** Scrolls down with the wheel, from the middle of the page, as far as it goes. */
async function scrollToEnd(page: Page): Promise<ScrollResult> {
  const before = await readLayout(page);
  const viewport = page.viewportSize() ?? { width: 0, height: 0 };
  await page.mouse.move(viewport.width / 2, before.outletBottom / 2);
  for (let step = 0; step < WHEEL_STEPS; step++) {
    await page.mouse.wheel(0, WHEEL_DELTA_PX);
  }
  await page.waitForTimeout(500);
  const after = await readLayout(page);
  const hidden = Math.max(0, Math.round(after.contentBottom - after.outletBottom));
  return {
    contentOverflows: before.contentBottom > before.outletBottom + 1,
    reachedBottom: hidden <= 1,
    hiddenAfterScrollPx: hidden,
  };
}

/** Appends a spacer taller than any screen at the end of the page's content. */
function stretchContent(page: Page): Promise<void> {
  return page.evaluate(
    ([selector, height]) => {
      const host = document.querySelector(selector);
      const container = host?.querySelector('ion-content, main') ?? host;
      const spacer = document.createElement('div');
      spacer.style.height = `${height}px`;
      container?.append(spacer);
    },
    [ROUTED_PAGE, STRETCH_PX] as const,
  );
}

/** Opens `path` and scrolls it to its end, then again with its content stretched. */
export async function measureScroll(page: Page, path: string): Promise<ScrollMeasure> {
  await openRoute(page, path);
  const layout = await readLayout(page);
  const natural = await scrollToEnd(page);
  await openRoute(page, path);
  await stretchContent(page);
  const stretched = await scrollToEnd(page);
  return {
    path,
    finalPath: new URL(page.url()).pathname,
    hasIonContent: layout.hasIonContent,
    pageOverflow: layout.pageOverflow,
    natural,
    stretched,
  };
}
