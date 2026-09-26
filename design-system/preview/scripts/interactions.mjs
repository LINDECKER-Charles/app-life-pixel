import { i18n } from './i18n.mjs';

const FEEDBACK_DURATION = 6000;
let toastTimer;
let selectedTool = 'pencil';

function showFeedback(key) {
  const toast = document.querySelector('#toast');
  clearTimeout(toastTimer);
  toast.textContent = i18n.translate(key);
  toast.hidden = false;
  toastTimer = setTimeout(() => {
    toast.hidden = true;
  }, FEEDBACK_DURATION);
}

function setGroupSelection(button, selector) {
  document.querySelectorAll(selector).forEach((item) => {
    item.setAttribute('aria-pressed', String(item === button));
  });
}

function setTool(button) {
  selectedTool = button.dataset.tool;
  setGroupSelection(button, '[data-tool]');
  const label = document.querySelector('#selected-tool');
  label.dataset.i18n = `tool.${selectedTool}`;
  label.textContent = i18n.translate(`tool.${selectedTool}`);
}

function setupEditor() {
  document.querySelectorAll('[data-tool]').forEach((button) => {
    button.addEventListener('click', () => setTool(button));
  });
  document.querySelectorAll('.swatch').forEach((button) => {
    button.addEventListener('click', () => setGroupSelection(button, '.swatch'));
  });
  document.querySelectorAll('[data-frame]').forEach((button) => {
    button.addEventListener('click', () => {
      setGroupSelection(button, '[data-frame]');
      document.querySelector('#canvas-pip').dataset.frame = button.dataset.frame;
    });
  });
  document.querySelector('#zoom').addEventListener('change', (event) => {
    document.querySelector('#canvas-pip').style.setProperty('--demo-zoom', event.target.value);
  });
  document.querySelector('#export-button').addEventListener('click', (event) => {
    const panel = document.querySelector('#export-options');
    panel.hidden = !panel.hidden;
    event.currentTarget.setAttribute('aria-expanded', String(!panel.hidden));
  });
}

function setupAppearance() {
  document.querySelector('#theme').addEventListener('click', (event) => {
    const isDark = document.documentElement.dataset.theme !== 'dark';
    document.documentElement.dataset.theme = isDark ? 'dark' : 'light';
    event.currentTarget.setAttribute('aria-pressed', String(isDark));
  });
  document.querySelector('#reduce-motion').addEventListener('change', (event) => {
    document.documentElement.dataset.motion = event.target.checked ? 'reduce' : 'system';
  });
  document.querySelector('#language').addEventListener('change', async (event) => {
    try {
      await i18n.setLanguage(event.target.value);
    } catch {
      event.target.value = document.documentElement.lang;
      showFeedback('feedback.language_error');
    }
  });
  document.addEventListener('cataloguechange', () => {
    document.querySelector('#toast').hidden = true;
  });
}

function submitCreation(event) {
  event.preventDefault();
  const input = document.querySelector('#animation-name');
  const isValid = input.value.trim().length > 0;
  document.querySelector('#name-error').hidden = isValid;
  input.setAttribute('aria-invalid', String(!isValid));
  if (!isValid) return input.focus();
  const name = document.querySelector('#demo-name');
  name.removeAttribute('data-i18n');
  name.textContent = input.value.trim();
  document.querySelector('#create-dialog').close();
  location.hash = 'studio';
  showFeedback('feedback.created');
}

function setupDialog() {
  const dialog = document.querySelector('#create-dialog');
  const input = document.querySelector('#animation-name');
  let opener;
  document.querySelectorAll('[data-open-dialog]').forEach((button) => {
    button.addEventListener('click', () => {
      opener = button;
      input.value = '';
      input.removeAttribute('aria-invalid');
      document.querySelector('#name-error').hidden = true;
      dialog.showModal();
      input.focus();
    });
  });
  dialog.addEventListener('close', () => opener?.focus());
  document.querySelector('#cancel-dialog').addEventListener('click', () => dialog.close());
  document.querySelector('#create-form').addEventListener('submit', submitCreation);
}

export function setupInteractions() {
  setupAppearance();
  setupEditor();
  setupDialog();
  document.querySelectorAll('[data-feedback]').forEach((button) => {
    button.addEventListener('click', () => showFeedback(button.dataset.feedback));
  });
}
