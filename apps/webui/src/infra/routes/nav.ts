import { type } from 'arktype';
import {
  BookOpen,
  Folders,
  KeyRound,
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
          to: '/subscriptions/manage',
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
        title: 'Credential',
        link: {
          to: '/credential3rd/manage',
        },
        icon: KeyRound,
        children: [
          {
            title: 'Manage',
            link: {
              to: '/credential3rd/manage',
            },
          },
          {
            title: 'Create',
            link: {
              to: '/credential3rd/create',
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

export const CreateCompleteAction = {
  Back: 'back',
  Detail: 'detail',
} as const;

export type CreateCompleteAction =
  (typeof CreateCompleteAction)[keyof typeof CreateCompleteAction];

export const CreateCompleteActionSchema = type.enumerated(
  ...Object.values(CreateCompleteAction)
);
