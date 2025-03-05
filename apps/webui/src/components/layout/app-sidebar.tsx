import {
  BookOpen,
  Bot,
  Folders,
  Settings2,
  SquareTerminal,
  Telescope,
} from 'lucide-solid';
import type { ComponentProps } from 'solid-js';
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
} from '~/components/ui/sidebar';
import { AppIcon } from './app-icon';
import { NavMain } from './nav-main';
import { NavUser } from './nav-user';

const navMain = [
  {
    group: 'Dashboard',
    items: [
      {
        title: 'Explore',
        link: {
          to: '/explore',
        },
        icon: Telescope,
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
        href: '#',
        icon: SquareTerminal,
        isActive: true,
        items: [
          {
            title: 'History',
            href: '#',
          },
          {
            title: 'Starred',
            href: '#',
          },
          {
            title: 'Settings',
            href: '#',
          },
        ],
      },
      {
        title: 'Models',
        href: '#',
        icon: Bot,
        items: [
          {
            title: 'Genesis',
            href: '#',
          },
          {
            title: 'Explorer',
            href: '#',
          },
          {
            title: 'Quantum',
            href: '#',
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
        href: '#',
        icon: Settings2,
        items: [
          {
            title: 'General',
            href: '#',
          },
          {
            title: 'Team',
            href: '#',
          },
          {
            title: 'Billing',
            href: '#',
          },
          {
            title: 'Limits',
            href: '#',
          },
        ],
      },
    ],
  },
];

const data = {
  user: {
    name: 'shadcn',
    email: 'm@example.com',
    avatar: '/avatars/shadcn.jpg',
  },
};

type AppSidebarRootProps = Omit<ComponentProps<typeof Sidebar>, 'collapsible'>;

export const AppSidebar = (props: AppSidebarRootProps) => {
  return (
    <Sidebar collapsible="icon" {...props}>
      <SidebarHeader>
        <AppIcon />
      </SidebarHeader>
      <SidebarContent>
        <NavMain groups={navMain} />
      </SidebarContent>
      <SidebarFooter>
        <NavUser user={data.user} />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  );
};
