import { chooseLanguage, SOURCE_LANGUAGE } from './choose-language';

describe('chooseLanguage', () => {
  const available = ['en', 'fr'];

  it('picks the first candidate that has a catalogue', () => {
    expect(chooseLanguage([undefined, 'de', 'fr', 'en'], available)).toBe('fr');
  });

  it('matches a regional code on its primary subtag, whatever its case', () => {
    expect(chooseLanguage(['FR-ca'], available)).toBe('fr');
  });

  it('falls back to the source language', () => {
    expect(chooseLanguage(['de-DE', undefined], available)).toBe(SOURCE_LANGUAGE);
    expect(chooseLanguage([], [])).toBe(SOURCE_LANGUAGE);
  });
});
