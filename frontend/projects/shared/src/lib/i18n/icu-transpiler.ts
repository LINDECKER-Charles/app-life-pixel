import { Injectable } from '@angular/core';
import { DefaultTranspiler, TranspileParams } from '@jsverse/transloco';
import { parse, Token } from '@messageformat/parser';
import { SOURCE_LANGUAGE } from './choose-language';
import { formatIcu } from './icu-format';

/**
 * Transloco's transpiler for ICU messages, in place of `@jsverse/transloco-messageformat`, whose
 * compiled messages need `eval` and fail under the desktop app's CSP. Transloco's own `{{ }}`
 * interpolation runs first, as with messageformat; each message is parsed once, with the parser
 * the catalogue check uses, then formatted for the active language.
 */
@Injectable()
export class IcuTranspiler extends DefaultTranspiler {
  private readonly parsed = new Map<string, readonly Token[]>();
  private locale = SOURCE_LANGUAGE;

  override transpile(params: TranspileParams): unknown {
    const value: unknown = super.transpile(params);
    return typeof value === 'string' && value !== '' ? this.format(value, params.params) : value;
  }

  onLangChanged(lang: string): void {
    this.locale = lang;
  }

  private format(message: string, params: TranspileParams['params'] = {}): string {
    try {
      return formatIcu(this.parse(message), params, this.locale);
    } catch (error: unknown) {
      console.error(`The ICU message "${message}" could not be formatted.`, error);
      return message;
    }
  }

  private parse(message: string): readonly Token[] {
    let tokens = this.parsed.get(message);
    if (tokens === undefined) {
      tokens = parse(message);
      this.parsed.set(message, tokens);
    }
    return tokens;
  }
}
