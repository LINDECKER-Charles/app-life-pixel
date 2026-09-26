import { Routes } from '@angular/router';
import { requireAccount } from './account/require-account';

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
    path: 'sign-in',
    title: 'auth.sign_in.title',
    loadComponent: () => import('./account/sign-in-page').then((m) => m.SignInPage),
  },
  {
    path: 'sign-up',
    title: 'auth.sign_up.title',
    loadComponent: () => import('./account/sign-up-page').then((m) => m.SignUpPage),
  },
  {
    path: 'verify-email',
    title: 'auth.verify_email.title',
    loadComponent: () => import('./account/verify-email-page').then((m) => m.VerifyEmailPage),
  },
  {
    path: 'reset-password',
    title: 'auth.reset_password.title',
    loadComponent: () => import('./account/reset-password-page').then((m) => m.ResetPasswordPage),
  },
  {
    path: 'reset-password/confirm',
    title: 'auth.reset_password_confirm.title',
    loadComponent: () =>
      import('./account/reset-password-confirm-page').then((m) => m.ResetPasswordConfirmPage),
  },
  {
    path: 'account',
    title: 'account.title',
    canActivate: [requireAccount],
    loadComponent: () => import('./account/account-page').then((m) => m.AccountPage),
  },
  {
    path: 'support',
    title: 'support.title',
    loadComponent: () => import('./support/support-page').then((m) => m.SupportPage),
  },
  {
    path: 'support/:requestId',
    title: 'support.thread.title',
    loadComponent: () => import('./support/support-request-page').then((m) => m.SupportRequestPage),
  },
  {
    path: '**',
    title: 'not_found.title',
    loadComponent: () => import('./not-found/not-found-page').then((m) => m.NotFoundPage),
  },
];
