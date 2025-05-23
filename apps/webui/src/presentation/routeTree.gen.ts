/* eslint-disable */

// @ts-nocheck

// noinspection JSUnusedGlobalSymbols

// This file was automatically generated by TanStack Router.
// You should NOT make any changes in this file as it will be overwritten.
// Additionally, you should also exclude this file from your linter and/or formatter to prevent it from being checked or modified.

// Import Routes

import { Route as R404Import } from './routes/404.tsx';
import { Route as rootRoute } from './routes/__root.tsx';
import { Route as AppExploreExploreImport } from './routes/_app/_explore/explore.tsx';
import { Route as AppExploreFeedImport } from './routes/_app/_explore/feed.tsx';
import { Route as AppBangumiManageImport } from './routes/_app/bangumi/manage.tsx';
import { Route as AppBangumiRouteImport } from './routes/_app/bangumi/route.tsx';
import { Route as AppPlaygroundGraphqlApiImport } from './routes/_app/playground/graphql-api.tsx';
import { Route as AppPlaygroundRouteImport } from './routes/_app/playground/route.tsx';
import { Route as AppRouteImport } from './routes/_app/route.tsx';
import { Route as AppSettingsDownloaderImport } from './routes/_app/settings/downloader.tsx';
import { Route as AppSettingsRouteImport } from './routes/_app/settings/route.tsx';
import { Route as AppSubscriptionsCreateImport } from './routes/_app/subscriptions/create.tsx';
import { Route as AppSubscriptionsDetailSubscriptionIdImport } from './routes/_app/subscriptions/detail.$subscriptionId.tsx';
import { Route as AppSubscriptionsEditSubscriptionIdImport } from './routes/_app/subscriptions/edit.$subscriptionId.tsx';
import { Route as AppSubscriptionsManageImport } from './routes/_app/subscriptions/manage.tsx';
import { Route as AppSubscriptionsRouteImport } from './routes/_app/subscriptions/route.tsx';
import { Route as AboutImport } from './routes/about.tsx';
import { Route as AuthOidcCallbackImport } from './routes/auth/oidc/callback.tsx';
import { Route as AuthSignInImport } from './routes/auth/sign-in.tsx';
import { Route as AuthSignUpImport } from './routes/auth/sign-up.tsx';
import { Route as IndexImport } from './routes/index.tsx';

// Create/Update Routes

const AboutRoute = AboutImport.update({
  id: '/about',
  path: '/about',
  getParentRoute: () => rootRoute,
} as any);

const R404Route = R404Import.update({
  id: '/404',
  path: '/404',
  getParentRoute: () => rootRoute,
} as any);

const AppRouteRoute = AppRouteImport.update({
  id: '/_app',
  getParentRoute: () => rootRoute,
} as any);

const IndexRoute = IndexImport.update({
  id: '/',
  path: '/',
  getParentRoute: () => rootRoute,
} as any);

const AuthSignUpRoute = AuthSignUpImport.update({
  id: '/auth/sign-up',
  path: '/auth/sign-up',
  getParentRoute: () => rootRoute,
} as any);

const AuthSignInRoute = AuthSignInImport.update({
  id: '/auth/sign-in',
  path: '/auth/sign-in',
  getParentRoute: () => rootRoute,
} as any);

const AppSubscriptionsRouteRoute = AppSubscriptionsRouteImport.update({
  id: '/subscriptions',
  path: '/subscriptions',
  getParentRoute: () => AppRouteRoute,
} as any);

const AppSettingsRouteRoute = AppSettingsRouteImport.update({
  id: '/settings',
  path: '/settings',
  getParentRoute: () => AppRouteRoute,
} as any);

const AppPlaygroundRouteRoute = AppPlaygroundRouteImport.update({
  id: '/playground',
  path: '/playground',
  getParentRoute: () => AppRouteRoute,
} as any);

const AppBangumiRouteRoute = AppBangumiRouteImport.update({
  id: '/bangumi',
  path: '/bangumi',
  getParentRoute: () => AppRouteRoute,
} as any);

const AuthOidcCallbackRoute = AuthOidcCallbackImport.update({
  id: '/auth/oidc/callback',
  path: '/auth/oidc/callback',
  getParentRoute: () => rootRoute,
} as any);

const AppSubscriptionsManageRoute = AppSubscriptionsManageImport.update({
  id: '/manage',
  path: '/manage',
  getParentRoute: () => AppSubscriptionsRouteRoute,
} as any);

const AppSubscriptionsCreateRoute = AppSubscriptionsCreateImport.update({
  id: '/create',
  path: '/create',
  getParentRoute: () => AppSubscriptionsRouteRoute,
} as any);

const AppSettingsDownloaderRoute = AppSettingsDownloaderImport.update({
  id: '/downloader',
  path: '/downloader',
  getParentRoute: () => AppSettingsRouteRoute,
} as any);

const AppPlaygroundGraphqlApiRoute = AppPlaygroundGraphqlApiImport.update({
  id: '/graphql-api',
  path: '/graphql-api',
  getParentRoute: () => AppPlaygroundRouteRoute,
} as any).lazy(() =>
  import('./routes/_app/playground/graphql-api.lazy.tsx').then((d) => d.Route)
);

const AppBangumiManageRoute = AppBangumiManageImport.update({
  id: '/manage',
  path: '/manage',
  getParentRoute: () => AppBangumiRouteRoute,
} as any);

const AppExploreFeedRoute = AppExploreFeedImport.update({
  id: '/_explore/feed',
  path: '/feed',
  getParentRoute: () => AppRouteRoute,
} as any);

const AppExploreExploreRoute = AppExploreExploreImport.update({
  id: '/_explore/explore',
  path: '/explore',
  getParentRoute: () => AppRouteRoute,
} as any);

const AppSubscriptionsEditSubscriptionIdRoute =
  AppSubscriptionsEditSubscriptionIdImport.update({
    id: '/edit/$subscriptionId',
    path: '/edit/$subscriptionId',
    getParentRoute: () => AppSubscriptionsRouteRoute,
  } as any);

const AppSubscriptionsDetailSubscriptionIdRoute =
  AppSubscriptionsDetailSubscriptionIdImport.update({
    id: '/detail/$subscriptionId',
    path: '/detail/$subscriptionId',
    getParentRoute: () => AppSubscriptionsRouteRoute,
  } as any);

// Populate the FileRoutesByPath interface

declare module '@tanstack/react-router' {
  interface FileRoutesByPath {
    '/': {
      id: '/';
      path: '/';
      fullPath: '/';
      preLoaderRoute: typeof IndexImport;
      parentRoute: typeof rootRoute;
    };
    '/_app': {
      id: '/_app';
      path: '';
      fullPath: '';
      preLoaderRoute: typeof AppRouteImport;
      parentRoute: typeof rootRoute;
    };
    '/404': {
      id: '/404';
      path: '/404';
      fullPath: '/404';
      preLoaderRoute: typeof R404Import;
      parentRoute: typeof rootRoute;
    };
    '/about': {
      id: '/about';
      path: '/about';
      fullPath: '/about';
      preLoaderRoute: typeof AboutImport;
      parentRoute: typeof rootRoute;
    };
    '/_app/bangumi': {
      id: '/_app/bangumi';
      path: '/bangumi';
      fullPath: '/bangumi';
      preLoaderRoute: typeof AppBangumiRouteImport;
      parentRoute: typeof AppRouteImport;
    };
    '/_app/playground': {
      id: '/_app/playground';
      path: '/playground';
      fullPath: '/playground';
      preLoaderRoute: typeof AppPlaygroundRouteImport;
      parentRoute: typeof AppRouteImport;
    };
    '/_app/settings': {
      id: '/_app/settings';
      path: '/settings';
      fullPath: '/settings';
      preLoaderRoute: typeof AppSettingsRouteImport;
      parentRoute: typeof AppRouteImport;
    };
    '/_app/subscriptions': {
      id: '/_app/subscriptions';
      path: '/subscriptions';
      fullPath: '/subscriptions';
      preLoaderRoute: typeof AppSubscriptionsRouteImport;
      parentRoute: typeof AppRouteImport;
    };
    '/auth/sign-in': {
      id: '/auth/sign-in';
      path: '/auth/sign-in';
      fullPath: '/auth/sign-in';
      preLoaderRoute: typeof AuthSignInImport;
      parentRoute: typeof rootRoute;
    };
    '/auth/sign-up': {
      id: '/auth/sign-up';
      path: '/auth/sign-up';
      fullPath: '/auth/sign-up';
      preLoaderRoute: typeof AuthSignUpImport;
      parentRoute: typeof rootRoute;
    };
    '/_app/_explore/explore': {
      id: '/_app/_explore/explore';
      path: '/explore';
      fullPath: '/explore';
      preLoaderRoute: typeof AppExploreExploreImport;
      parentRoute: typeof AppRouteImport;
    };
    '/_app/_explore/feed': {
      id: '/_app/_explore/feed';
      path: '/feed';
      fullPath: '/feed';
      preLoaderRoute: typeof AppExploreFeedImport;
      parentRoute: typeof AppRouteImport;
    };
    '/_app/bangumi/manage': {
      id: '/_app/bangumi/manage';
      path: '/manage';
      fullPath: '/bangumi/manage';
      preLoaderRoute: typeof AppBangumiManageImport;
      parentRoute: typeof AppBangumiRouteImport;
    };
    '/_app/playground/graphql-api': {
      id: '/_app/playground/graphql-api';
      path: '/graphql-api';
      fullPath: '/playground/graphql-api';
      preLoaderRoute: typeof AppPlaygroundGraphqlApiImport;
      parentRoute: typeof AppPlaygroundRouteImport;
    };
    '/_app/settings/downloader': {
      id: '/_app/settings/downloader';
      path: '/downloader';
      fullPath: '/settings/downloader';
      preLoaderRoute: typeof AppSettingsDownloaderImport;
      parentRoute: typeof AppSettingsRouteImport;
    };
    '/_app/subscriptions/create': {
      id: '/_app/subscriptions/create';
      path: '/create';
      fullPath: '/subscriptions/create';
      preLoaderRoute: typeof AppSubscriptionsCreateImport;
      parentRoute: typeof AppSubscriptionsRouteImport;
    };
    '/_app/subscriptions/manage': {
      id: '/_app/subscriptions/manage';
      path: '/manage';
      fullPath: '/subscriptions/manage';
      preLoaderRoute: typeof AppSubscriptionsManageImport;
      parentRoute: typeof AppSubscriptionsRouteImport;
    };
    '/auth/oidc/callback': {
      id: '/auth/oidc/callback';
      path: '/auth/oidc/callback';
      fullPath: '/auth/oidc/callback';
      preLoaderRoute: typeof AuthOidcCallbackImport;
      parentRoute: typeof rootRoute;
    };
    '/_app/subscriptions/detail/$subscriptionId': {
      id: '/_app/subscriptions/detail/$subscriptionId';
      path: '/detail/$subscriptionId';
      fullPath: '/subscriptions/detail/$subscriptionId';
      preLoaderRoute: typeof AppSubscriptionsDetailSubscriptionIdImport;
      parentRoute: typeof AppSubscriptionsRouteImport;
    };
    '/_app/subscriptions/edit/$subscriptionId': {
      id: '/_app/subscriptions/edit/$subscriptionId';
      path: '/edit/$subscriptionId';
      fullPath: '/subscriptions/edit/$subscriptionId';
      preLoaderRoute: typeof AppSubscriptionsEditSubscriptionIdImport;
      parentRoute: typeof AppSubscriptionsRouteImport;
    };
  }
}

// Create and export the route tree

interface AppBangumiRouteRouteChildren {
  AppBangumiManageRoute: typeof AppBangumiManageRoute;
}

const AppBangumiRouteRouteChildren: AppBangumiRouteRouteChildren = {
  AppBangumiManageRoute: AppBangumiManageRoute,
};

const AppBangumiRouteRouteWithChildren = AppBangumiRouteRoute._addFileChildren(
  AppBangumiRouteRouteChildren
);

interface AppPlaygroundRouteRouteChildren {
  AppPlaygroundGraphqlApiRoute: typeof AppPlaygroundGraphqlApiRoute;
}

const AppPlaygroundRouteRouteChildren: AppPlaygroundRouteRouteChildren = {
  AppPlaygroundGraphqlApiRoute: AppPlaygroundGraphqlApiRoute,
};

const AppPlaygroundRouteRouteWithChildren =
  AppPlaygroundRouteRoute._addFileChildren(AppPlaygroundRouteRouteChildren);

interface AppSettingsRouteRouteChildren {
  AppSettingsDownloaderRoute: typeof AppSettingsDownloaderRoute;
}

const AppSettingsRouteRouteChildren: AppSettingsRouteRouteChildren = {
  AppSettingsDownloaderRoute: AppSettingsDownloaderRoute,
};

const AppSettingsRouteRouteWithChildren =
  AppSettingsRouteRoute._addFileChildren(AppSettingsRouteRouteChildren);

interface AppSubscriptionsRouteRouteChildren {
  AppSubscriptionsCreateRoute: typeof AppSubscriptionsCreateRoute;
  AppSubscriptionsManageRoute: typeof AppSubscriptionsManageRoute;
  AppSubscriptionsDetailSubscriptionIdRoute: typeof AppSubscriptionsDetailSubscriptionIdRoute;
  AppSubscriptionsEditSubscriptionIdRoute: typeof AppSubscriptionsEditSubscriptionIdRoute;
}

const AppSubscriptionsRouteRouteChildren: AppSubscriptionsRouteRouteChildren = {
  AppSubscriptionsCreateRoute: AppSubscriptionsCreateRoute,
  AppSubscriptionsManageRoute: AppSubscriptionsManageRoute,
  AppSubscriptionsDetailSubscriptionIdRoute:
    AppSubscriptionsDetailSubscriptionIdRoute,
  AppSubscriptionsEditSubscriptionIdRoute:
    AppSubscriptionsEditSubscriptionIdRoute,
};

const AppSubscriptionsRouteRouteWithChildren =
  AppSubscriptionsRouteRoute._addFileChildren(
    AppSubscriptionsRouteRouteChildren
  );

interface AppRouteRouteChildren {
  AppBangumiRouteRoute: typeof AppBangumiRouteRouteWithChildren;
  AppPlaygroundRouteRoute: typeof AppPlaygroundRouteRouteWithChildren;
  AppSettingsRouteRoute: typeof AppSettingsRouteRouteWithChildren;
  AppSubscriptionsRouteRoute: typeof AppSubscriptionsRouteRouteWithChildren;
  AppExploreExploreRoute: typeof AppExploreExploreRoute;
  AppExploreFeedRoute: typeof AppExploreFeedRoute;
}

const AppRouteRouteChildren: AppRouteRouteChildren = {
  AppBangumiRouteRoute: AppBangumiRouteRouteWithChildren,
  AppPlaygroundRouteRoute: AppPlaygroundRouteRouteWithChildren,
  AppSettingsRouteRoute: AppSettingsRouteRouteWithChildren,
  AppSubscriptionsRouteRoute: AppSubscriptionsRouteRouteWithChildren,
  AppExploreExploreRoute: AppExploreExploreRoute,
  AppExploreFeedRoute: AppExploreFeedRoute,
};

const AppRouteRouteWithChildren = AppRouteRoute._addFileChildren(
  AppRouteRouteChildren
);

export interface FileRoutesByFullPath {
  '/': typeof IndexRoute;
  '': typeof AppRouteRouteWithChildren;
  '/404': typeof R404Route;
  '/about': typeof AboutRoute;
  '/bangumi': typeof AppBangumiRouteRouteWithChildren;
  '/playground': typeof AppPlaygroundRouteRouteWithChildren;
  '/settings': typeof AppSettingsRouteRouteWithChildren;
  '/subscriptions': typeof AppSubscriptionsRouteRouteWithChildren;
  '/auth/sign-in': typeof AuthSignInRoute;
  '/auth/sign-up': typeof AuthSignUpRoute;
  '/explore': typeof AppExploreExploreRoute;
  '/feed': typeof AppExploreFeedRoute;
  '/bangumi/manage': typeof AppBangumiManageRoute;
  '/playground/graphql-api': typeof AppPlaygroundGraphqlApiRoute;
  '/settings/downloader': typeof AppSettingsDownloaderRoute;
  '/subscriptions/create': typeof AppSubscriptionsCreateRoute;
  '/subscriptions/manage': typeof AppSubscriptionsManageRoute;
  '/auth/oidc/callback': typeof AuthOidcCallbackRoute;
  '/subscriptions/detail/$subscriptionId': typeof AppSubscriptionsDetailSubscriptionIdRoute;
  '/subscriptions/edit/$subscriptionId': typeof AppSubscriptionsEditSubscriptionIdRoute;
}

export interface FileRoutesByTo {
  '/': typeof IndexRoute;
  '': typeof AppRouteRouteWithChildren;
  '/404': typeof R404Route;
  '/about': typeof AboutRoute;
  '/bangumi': typeof AppBangumiRouteRouteWithChildren;
  '/playground': typeof AppPlaygroundRouteRouteWithChildren;
  '/settings': typeof AppSettingsRouteRouteWithChildren;
  '/subscriptions': typeof AppSubscriptionsRouteRouteWithChildren;
  '/auth/sign-in': typeof AuthSignInRoute;
  '/auth/sign-up': typeof AuthSignUpRoute;
  '/explore': typeof AppExploreExploreRoute;
  '/feed': typeof AppExploreFeedRoute;
  '/bangumi/manage': typeof AppBangumiManageRoute;
  '/playground/graphql-api': typeof AppPlaygroundGraphqlApiRoute;
  '/settings/downloader': typeof AppSettingsDownloaderRoute;
  '/subscriptions/create': typeof AppSubscriptionsCreateRoute;
  '/subscriptions/manage': typeof AppSubscriptionsManageRoute;
  '/auth/oidc/callback': typeof AuthOidcCallbackRoute;
  '/subscriptions/detail/$subscriptionId': typeof AppSubscriptionsDetailSubscriptionIdRoute;
  '/subscriptions/edit/$subscriptionId': typeof AppSubscriptionsEditSubscriptionIdRoute;
}

export interface FileRoutesById {
  __root__: typeof rootRoute;
  '/': typeof IndexRoute;
  '/_app': typeof AppRouteRouteWithChildren;
  '/404': typeof R404Route;
  '/about': typeof AboutRoute;
  '/_app/bangumi': typeof AppBangumiRouteRouteWithChildren;
  '/_app/playground': typeof AppPlaygroundRouteRouteWithChildren;
  '/_app/settings': typeof AppSettingsRouteRouteWithChildren;
  '/_app/subscriptions': typeof AppSubscriptionsRouteRouteWithChildren;
  '/auth/sign-in': typeof AuthSignInRoute;
  '/auth/sign-up': typeof AuthSignUpRoute;
  '/_app/_explore/explore': typeof AppExploreExploreRoute;
  '/_app/_explore/feed': typeof AppExploreFeedRoute;
  '/_app/bangumi/manage': typeof AppBangumiManageRoute;
  '/_app/playground/graphql-api': typeof AppPlaygroundGraphqlApiRoute;
  '/_app/settings/downloader': typeof AppSettingsDownloaderRoute;
  '/_app/subscriptions/create': typeof AppSubscriptionsCreateRoute;
  '/_app/subscriptions/manage': typeof AppSubscriptionsManageRoute;
  '/auth/oidc/callback': typeof AuthOidcCallbackRoute;
  '/_app/subscriptions/detail/$subscriptionId': typeof AppSubscriptionsDetailSubscriptionIdRoute;
  '/_app/subscriptions/edit/$subscriptionId': typeof AppSubscriptionsEditSubscriptionIdRoute;
}

export interface FileRouteTypes {
  fileRoutesByFullPath: FileRoutesByFullPath;
  fullPaths:
    | '/'
    | ''
    | '/404'
    | '/about'
    | '/bangumi'
    | '/playground'
    | '/settings'
    | '/subscriptions'
    | '/auth/sign-in'
    | '/auth/sign-up'
    | '/explore'
    | '/feed'
    | '/bangumi/manage'
    | '/playground/graphql-api'
    | '/settings/downloader'
    | '/subscriptions/create'
    | '/subscriptions/manage'
    | '/auth/oidc/callback'
    | '/subscriptions/detail/$subscriptionId'
    | '/subscriptions/edit/$subscriptionId';
  fileRoutesByTo: FileRoutesByTo;
  to:
    | '/'
    | ''
    | '/404'
    | '/about'
    | '/bangumi'
    | '/playground'
    | '/settings'
    | '/subscriptions'
    | '/auth/sign-in'
    | '/auth/sign-up'
    | '/explore'
    | '/feed'
    | '/bangumi/manage'
    | '/playground/graphql-api'
    | '/settings/downloader'
    | '/subscriptions/create'
    | '/subscriptions/manage'
    | '/auth/oidc/callback'
    | '/subscriptions/detail/$subscriptionId'
    | '/subscriptions/edit/$subscriptionId';
  id:
    | '__root__'
    | '/'
    | '/_app'
    | '/404'
    | '/about'
    | '/_app/bangumi'
    | '/_app/playground'
    | '/_app/settings'
    | '/_app/subscriptions'
    | '/auth/sign-in'
    | '/auth/sign-up'
    | '/_app/_explore/explore'
    | '/_app/_explore/feed'
    | '/_app/bangumi/manage'
    | '/_app/playground/graphql-api'
    | '/_app/settings/downloader'
    | '/_app/subscriptions/create'
    | '/_app/subscriptions/manage'
    | '/auth/oidc/callback'
    | '/_app/subscriptions/detail/$subscriptionId'
    | '/_app/subscriptions/edit/$subscriptionId';
  fileRoutesById: FileRoutesById;
}

export interface RootRouteChildren {
  IndexRoute: typeof IndexRoute;
  AppRouteRoute: typeof AppRouteRouteWithChildren;
  R404Route: typeof R404Route;
  AboutRoute: typeof AboutRoute;
  AuthSignInRoute: typeof AuthSignInRoute;
  AuthSignUpRoute: typeof AuthSignUpRoute;
  AuthOidcCallbackRoute: typeof AuthOidcCallbackRoute;
}

const rootRouteChildren: RootRouteChildren = {
  IndexRoute: IndexRoute,
  AppRouteRoute: AppRouteRouteWithChildren,
  R404Route: R404Route,
  AboutRoute: AboutRoute,
  AuthSignInRoute: AuthSignInRoute,
  AuthSignUpRoute: AuthSignUpRoute,
  AuthOidcCallbackRoute: AuthOidcCallbackRoute,
};

export const routeTree = rootRoute
  ._addFileChildren(rootRouteChildren)
  ._addFileTypes<FileRouteTypes>();

/* ROUTE_MANIFEST_START
{
  "routes": {
    "__root__": {
      "filePath": "__root.tsx",
      "children": [
        "/",
        "/_app",
        "/404",
        "/about",
        "/auth/sign-in",
        "/auth/sign-up",
        "/auth/oidc/callback"
      ]
    },
    "/": {
      "filePath": "index.tsx"
    },
    "/_app": {
      "filePath": "_app/route.tsx",
      "children": [
        "/_app/bangumi",
        "/_app/playground",
        "/_app/settings",
        "/_app/subscriptions",
        "/_app/_explore/explore",
        "/_app/_explore/feed"
      ]
    },
    "/404": {
      "filePath": "404.tsx"
    },
    "/about": {
      "filePath": "about.tsx"
    },
    "/_app/bangumi": {
      "filePath": "_app/bangumi/route.tsx",
      "parent": "/_app",
      "children": [
        "/_app/bangumi/manage"
      ]
    },
    "/_app/playground": {
      "filePath": "_app/playground/route.tsx",
      "parent": "/_app",
      "children": [
        "/_app/playground/graphql-api"
      ]
    },
    "/_app/settings": {
      "filePath": "_app/settings/route.tsx",
      "parent": "/_app",
      "children": [
        "/_app/settings/downloader"
      ]
    },
    "/_app/subscriptions": {
      "filePath": "_app/subscriptions/route.tsx",
      "parent": "/_app",
      "children": [
        "/_app/subscriptions/create",
        "/_app/subscriptions/manage",
        "/_app/subscriptions/detail/$subscriptionId",
        "/_app/subscriptions/edit/$subscriptionId"
      ]
    },
    "/auth/sign-in": {
      "filePath": "auth/sign-in.tsx"
    },
    "/auth/sign-up": {
      "filePath": "auth/sign-up.tsx"
    },
    "/_app/_explore/explore": {
      "filePath": "_app/_explore/explore.tsx",
      "parent": "/_app"
    },
    "/_app/_explore/feed": {
      "filePath": "_app/_explore/feed.tsx",
      "parent": "/_app"
    },
    "/_app/bangumi/manage": {
      "filePath": "_app/bangumi/manage.tsx",
      "parent": "/_app/bangumi"
    },
    "/_app/playground/graphql-api": {
      "filePath": "_app/playground/graphql-api.tsx",
      "parent": "/_app/playground"
    },
    "/_app/settings/downloader": {
      "filePath": "_app/settings/downloader.tsx",
      "parent": "/_app/settings"
    },
    "/_app/subscriptions/create": {
      "filePath": "_app/subscriptions/create.tsx",
      "parent": "/_app/subscriptions"
    },
    "/_app/subscriptions/manage": {
      "filePath": "_app/subscriptions/manage.tsx",
      "parent": "/_app/subscriptions"
    },
    "/auth/oidc/callback": {
      "filePath": "auth/oidc/callback.tsx"
    },
    "/_app/subscriptions/detail/$subscriptionId": {
      "filePath": "_app/subscriptions/detail.$subscriptionId.tsx",
      "parent": "/_app/subscriptions"
    },
    "/_app/subscriptions/edit/$subscriptionId": {
      "filePath": "_app/subscriptions/edit.$subscriptionId.tsx",
      "parent": "/_app/subscriptions"
    }
  }
}
ROUTE_MANIFEST_END */
