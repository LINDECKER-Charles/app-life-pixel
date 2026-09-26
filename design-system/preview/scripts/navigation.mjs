const DEFAULT_SECTION = 'overview';

function selectSection(shouldFocus) {
  const requested = location.hash.slice(1);
  if (requested === 'main') return;
  const sections = [...document.querySelectorAll('main > section')];
  const selected = sections.find((section) => section.id === requested)?.id ?? DEFAULT_SECTION;
  sections.forEach((section) => {
    section.hidden = section.id !== selected;
  });
  document.querySelectorAll('[data-section]').forEach((link) => {
    if (link.dataset.section === selected) link.setAttribute('aria-current', 'page');
    else link.removeAttribute('aria-current');
  });
  if (!shouldFocus) return;
  const heading = document.querySelector(`#${selected} h1`);
  heading.tabIndex = -1;
  heading.focus({ preventScroll: true });
  window.scrollTo({ top: 0, behavior: 'instant' });
}

export function setupNavigation() {
  selectSection(false);
  window.addEventListener('hashchange', () => selectSection(true));
}
