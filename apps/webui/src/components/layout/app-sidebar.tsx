import type { ComponentProps } from 'solid-js';
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
} from '~/components/ui/sidebar';
import { AppNavMainData } from '~/config/app-layout';
import { AppIcon } from './app-icon';
import { NavMain } from './nav-main';
import { NavUser } from './nav-user';

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
        <NavMain groups={AppNavMainData} />
      </SidebarContent>
      <SidebarFooter>
        <NavUser user={data.user} />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  );
};
