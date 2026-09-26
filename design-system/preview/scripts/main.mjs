import { i18n } from './i18n.mjs';
import { setupNavigation } from './navigation.mjs';
import { setupInteractions } from './interactions.mjs';

const SECTIONS = ['overview', 'foundations', 'components', 'studio', 'journeys'];

async function loadSection(section) {
  const response = await fetch(`sections/${section}.html`);
  if (!response.ok) throw new Error(`Section request failed: ${response.status}`);
  return response.text();
}

async function start() {
  const sections = await Promise.all(SECTIONS.map(loadSection));
  const main = document.querySelector('main');
  // Only trusted, local, authored fragments; no user content enters this HTML insertion.
  main.innerHTML = sections.join('\n');
  await i18n.setLanguage('fr');
  setupNavigation();
  setupInteractions();
  main.setAttribute('aria-busy', 'false');
}

start().catch((error) => {
  console.error(error);
  const main = document.querySelector('main');
  main.setAttribute('aria-busy', 'false');
  main.setAttribute('role', 'alert');
  // Catalogue-independent bootstrap text remains a translation key if the catalogue is unavailable.
  main.textContent = i18n.translate('feedback.load_error');
});
