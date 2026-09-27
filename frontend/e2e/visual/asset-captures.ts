import { expect, type Page } from '@playwright/test';
import { captureAt, settle } from './screens';
import { VIEWPORTS, writeJson } from './visual-matrix';

/** Captures asset-bearing screens and records image loading and page overflow at every width. */
export class AssetCaptures {
  private readonly measurements: unknown[] = [];
  private readonly pageErrors: string[] = [];

  constructor(
    private readonly page: Page,
    private readonly folder: string,
  ) {
    page.on('pageerror', (error) => this.pageErrors.push(error.message));
  }

  async everyWidth(screen: string, focusSelector?: string): Promise<void> {
    for (const viewport of VIEWPORTS) {
      await this.page.setViewportSize(viewport);
      if (focusSelector) await this.page.locator(focusSelector).scrollIntoViewIfNeeded();
      await settle(this.page);
      const measurement = await this.measure();
      const screenshot = await captureAt(this.page, `${this.folder}/${screen}`, viewport);
      this.measurements.push({ screen, viewport, screenshot, ...measurement });
      expect(measurement.brokenImages, `${screen} at ${viewport.width}px`).toEqual([]);
      expect(measurement.scrollWidth, `${screen} must not scroll sideways`).toBeLessThanOrEqual(
        viewport.width,
      );
    }
    await this.page.setViewportSize(VIEWPORTS[0]);
  }

  report(): void {
    writeJson(`${this.folder}/report.json`, {
      measurements: this.measurements,
      pageErrors: this.pageErrors,
    });
    expect(this.pageErrors, 'uncaught errors on asset screens').toEqual([]);
  }

  private measure(): Promise<{ brokenImages: string[]; scrollWidth: number; images: string[] }> {
    return this.page.evaluate(() => {
      const images = [...document.images];
      return {
        images: images.map((image) => new URL(image.currentSrc || image.src).pathname),
        brokenImages: images
          .filter((image) => !image.complete || image.naturalWidth === 0)
          .map((image) => image.currentSrc || image.src),
        scrollWidth: document.documentElement.scrollWidth,
      };
    });
  }
}
