import {
  Outlet,
  createRootRouteWithContext,
  createRoute,
  createRouter,
  redirect,
} from '@tanstack/react-router';
import { Layout } from '../components/Layout';
import { BookDetailPage } from '../pages/BookDetailPage';
import { BooksPage } from '../pages/BooksPage';
import { DashboardPage } from '../pages/DashboardPage';
import { JobsPage } from '../pages/JobsPage';
import { LibrariesPage } from '../pages/LibrariesPage';
import { LoginPage } from '../pages/LoginPage';
import { ReaderPage } from '../pages/ReaderPage';
import { SettingsPage } from '../pages/SettingsPage';
import { ShelvesPage } from '../pages/ShelvesPage';
import { UploadPage } from '../pages/UploadPage';
import type { useAuth } from '../lib/auth';

type RouterContext = {
  auth: ReturnType<typeof useAuth>;
};

const rootRoute = createRootRouteWithContext<RouterContext>()({
  component: Outlet,
});

const loginRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/login',
  component: LoginPage,
});

const appRoute = createRoute({
  getParentRoute: () => rootRoute,
  id: 'app',
  beforeLoad: ({ context }) => {
    if (!context.auth.token && !localStorage.getItem('auth_token')) {
      throw redirect({ to: '/login' });
    }
  },
  component: () => (
    <Layout>
      <Outlet />
    </Layout>
  ),
});

const dashboardRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/',
  component: DashboardPage,
});

const booksRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/books',
  component: BooksPage,
});

const shelvesRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/shelves',
  component: ShelvesPage,
});

const shelfDetailRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/shelves/$shelfId',
  component: ShelvesPage,
});

const bookDetailRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/books/$bookId',
  component: BookDetailPage,
});

const readerRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/books/$bookId/read',
  component: ReaderPage,
});

const uploadRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/upload',
  component: UploadPage,
});

const librariesRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/libraries',
  component: LibrariesPage,
});

const jobsRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/jobs',
  component: JobsPage,
});

const settingsRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/settings',
  component: SettingsPage,
});

const routeTree = rootRoute.addChildren([
  loginRoute,
  appRoute.addChildren([
    dashboardRoute,
    booksRoute,
    shelvesRoute,
    shelfDetailRoute,
    bookDetailRoute,
    readerRoute,
    uploadRoute,
    librariesRoute,
    jobsRoute,
    settingsRoute,
  ]),
]);

export const router = createRouter({
  routeTree,
  context: {
    auth: undefined!,
  },
});

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}
