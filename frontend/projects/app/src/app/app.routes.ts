import { Routes } from '@angular/router';
import { requireAccount } from './account/require-account';
import { requireLibraryAccess } from './library/require-library-access';
import { desktopOnly, hostedOnly } from './platform/platform-guards';

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
    path: 'settings/agents',
    title: 'mcp.agents.title',
    canActivate: [desktopOnly],
    loadComponent: () => import('./mcp/agents-page').then((m) => m.AgentsPage),
  },
  {
    path: 'legal/:page',
    title: 'legal.title',
    loadComponent: () => import('./legal/legal-page').then((m) => m.LegalPage),
  },
  {
    path: 'sign-in',
    title: 'auth.sign_in.title',
    canActivate: [hostedOnly],
    loadComponent: () => import('./account/sign-in-page').then((m) => m.SignInPage),
  },
  {
    path: 'sign-up',
    title: 'auth.sign_up.title',
    canActivate: [hostedOnly],
    loadComponent: () => import('./account/sign-up-page').then((m) => m.SignUpPage),
  },
  {
    path: 'verify-email',
    title: 'auth.verify_email.title',
    canActivate: [hostedOnly],
    loadComponent: () => import('./account/verify-email-page').then((m) => m.VerifyEmailPage),
  },
  {
    path: 'reset-password',
    title: 'auth.reset_password.title',
    canActivate: [hostedOnly],
    loadComponent: () => import('./account/reset-password-page').then((m) => m.ResetPasswordPage),
  },
  {
    path: 'reset-password/confirm',
    title: 'auth.reset_password_confirm.title',
    canActivate: [hostedOnly],
    loadComponent: () =>
      import('./account/reset-password-confirm-page').then((m) => m.ResetPasswordConfirmPage),
  },
  {
    path: 'account',
    title: 'account.title',
    canActivate: [hostedOnly, requireAccount],
    loadComponent: () => import('./account/account-page').then((m) => m.AccountPage),
  },
  {
    path: 'library',
    title: 'library.title',
    canActivate: [requireLibraryAccess],
    loadComponent: () => import('./library/pages/library-page').then((m) => m.LibraryPage),
  },
  {
    path: 'library/projects/:projectId',
    title: 'library.title',
    canActivate: [requireLibraryAccess],
    loadComponent: () => import('./library/pages/project-page').then((m) => m.ProjectPage),
  },
  {
    path: 'support',
    title: 'support.title',
    canActivate: [hostedOnly],
    loadComponent: () => import('./support/support-page').then((m) => m.SupportPage),
  },
  {
    path: 'support/:requestId',
    title: 'support.thread.title',
    canActivate: [hostedOnly],
    loadComponent: () => import('./support/support-request-page').then((m) => m.SupportRequestPage),
  },
  {
    path: '**',
    title: 'not_found.title',
    loadComponent: () => import('./not-found/not-found-page').then((m) => m.NotFoundPage),
  },
];
