/** A language the catalogues offer, as `languages.json` lists it. */
export interface Language {
  /** The code of its catalogue: `en` for `en.json`. */
  readonly code: string;
  /** Its name in that language, shown as is in every language. */
  readonly name: string;
}
