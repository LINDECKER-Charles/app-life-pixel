import { Injectable } from '@angular/core';

/** The system's clipboard, written from a click: the webview's own, no plugin needed. */
@Injectable({ providedIn: 'root' })
export class Clipboard {
  copy(text: string): Promise<void> {
    return navigator.clipboard.writeText(text);
  }
}
