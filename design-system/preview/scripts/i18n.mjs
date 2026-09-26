const PREFIX = 'design_system.';
const SUPPORTED_LANGUAGES = ['en', 'fr'];
let catalogue = {};
let requestVersion = 0;

function translate(key) {
  return catalogue[`${PREFIX}${key}`] ?? `${PREFIX}${key}`;
}

function applyTranslation(root) {
  const attributes = [
    ['data-i18n', 'textContent'],
    ['data-i18n-label', 'aria-label'],
    ['data-i18n-placeholder', 'placeholder'],
  ];
  for (const [source, target] of attributes) {
    root.querySelectorAll(`[${source}]`).forEach((element) => {
      const text = translate(element.getAttribute(source));
      if (target === 'textContent') element.textContent = text;
      else element.setAttribute(target, text);
    });
  }
}

async function setLanguage(language) {
  const selected = SUPPORTED_LANGUAGES.includes(language) ? language : 'en';
  const version = ++requestVersion;
  const response = await fetch(`/i18n/${selected}.json`);
  if (!response.ok) throw new Error(`Catalogue request failed: ${response.status}`);
  const nextCatalogue = await response.json();
  if (version !== requestVersion) return;
  catalogue = nextCatalogue;
  document.documentElement.lang = selected;
  document.querySelector('#language').value = selected;
  document.title = `${translate('brand_caption')} · Life Pixel`;
  applyTranslation(document);
  document.dispatchEvent(new Event('cataloguechange'));
}

export const i18n = { translate, setLanguage };
