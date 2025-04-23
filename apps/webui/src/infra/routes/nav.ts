import {
  BookOpen,
  Folders,
  Settings2,
  SquareTerminal,
  Telescope,
} from 'lucide-react';

export const AppNavMainData = [
  {
    group: 'Dashboard',
    items: [
      {
        title: 'Explore',
        icon: Telescope,
        children: [
          {
            title: 'Feed',
            link: {
              to: '/feed',
            },
          },
          {
            title: 'Explore',
            link: {
              to: '/explore',
            },
          },
        ],
      },
      {
        title: 'Subscriptions',
        link: {
          to: '/subscriptions',
        },
        icon: Folders,
        children: [
          {
            title: 'Manage',
            link: {
              to: '/subscriptions/manage',
            },
          },
          {
            title: 'Create',
            link: {
              to: '/subscriptions/create',
            },
          },
        ],
      },
      {
        title: 'Playground',
        icon: SquareTerminal,
        link: {
          to: '/playground',
        },
        children: [
          {
            title: 'GraphQL Api',
            link: {
              to: '/playground/graphql-api',
            },
          },
        ],
      },
      {
        title: 'Documentation',
        link: {
          href: 'https://github.com/dumtruck/konobangu/wiki',
          target: '_blank',
        },
        icon: BookOpen,
      },
      {
        title: 'Settings',
        link: {
          to: '/settings',
        },
        icon: Settings2,
        children: [
          {
            title: 'Downloader',
            link: {
              to: '/settings/downloader',
            },
          },
        ],
      },
    ],
  },
];
