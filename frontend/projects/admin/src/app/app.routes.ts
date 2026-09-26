import { Routes } from '@angular/router';
import { requireAdmin, requireSignedOut } from './core/guards';

// One lazy route per page (support-admin.md, H12); a route's title is an i18n key. Every page but
// sign-in needs a session.
export const routes: Routes = [
  {
    path: 'sign-in',
    title: 'admin.sign_in.title',
    canActivate: [requireSignedOut],
    loadComponent: () => import('./sign-in/sign-in-page').then((m) => m.SignInPage),
  },
  {
    path: '',
    canActivateChild: [requireAdmin],
    children: [
      {
        path: '',
        pathMatch: 'full',
        title: 'admin.overview.title',
        loadComponent: () => import('./monitoring/overview-page').then((m) => m.OverviewPage),
      },
      {
        path: 'monitoring/:panel',
        title: 'admin.monitoring.title',
        loadComponent: () => import('./monitoring/panel-page').then((m) => m.PanelPage),
      },
      {
        path: 'logs',
        title: 'admin.logs.title',
        loadComponent: () => import('./monitoring/logs-page').then((m) => m.LogsPage),
      },
      {
        path: 'alerts',
        title: 'admin.alerts.title',
        loadComponent: () => import('./monitoring/alerts-page').then((m) => m.AlertsPage),
      },
      {
        path: 'users',
        title: 'admin.users.title',
        loadComponent: () => import('./users/users-page').then((m) => m.UsersPage),
      },
      {
        path: 'users/:id',
        title: 'admin.user.title',
        loadComponent: () => import('./users/user-page').then((m) => m.UserPage),
      },
      {
        path: 'support',
        title: 'admin.support.title',
        loadComponent: () => import('./support/support-queue-page').then((m) => m.SupportQueuePage),
      },
      {
        path: 'support/:id',
        title: 'admin.support.thread_title',
        loadComponent: () =>
          import('./support/support-thread-page').then((m) => m.SupportThreadPage),
      },
      {
        path: 'audit',
        title: 'admin.audit.title',
        loadComponent: () => import('./audit/audit-page').then((m) => m.AuditPage),
      },
      {
        path: '**',
        title: 'admin.not_found.title',
        loadComponent: () => import('./shell/not-found-page').then((m) => m.NotFoundPage),
      },
    ],
  },
];
