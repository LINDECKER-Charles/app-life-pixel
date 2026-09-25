import { Routes } from '@angular/router';

// One lazy route per feature; a route's title is an i18n key. The not-found route stays last.
export const routes: Routes = [
  { path: '', pathMatch: 'full', redirectTo: 'editor' },
  {
    path: 'editor',
    title: 'editor.heading',
    loadComponent: () => import('./editor/editor-page').then((m) => m.EditorPage),
  },
  {
    path: 'editor/:animationId',
    title: 'editor.heading',
    loadComponent: () => import('./editor/editor-page').then((m) => m.EditorPage),
  },
  {
    path: 'settings',
    title: 'settings.title',
    loadComponent: () => import('./settings/settings-page').then((m) => m.SettingsPage),
  },
  {
    path: 'legal/:page',
    title: 'legal.title',
    loadComponent: () => import('./legal/legal-page').then((m) => m.LegalPage),
  },
  {
    path: '**',
    title: 'not_found.title',
    loadComponent: () => import('./not-found/not-found-page').then((m) => m.NotFoundPage),
  },
];
