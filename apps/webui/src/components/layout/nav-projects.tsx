import { Link } from '@tanstack/solid-router';
import {
  Folder,
  Forward,
  type LucideIcon,
  MoreHorizontal,
  Trash2,
} from 'lucide-solid';
import { type ComponentProps, For } from 'solid-js';

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '~/components/ui/dropdown-menu';
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuAction,
  SidebarMenuButton,
  SidebarMenuItem,
} from '~/components/ui/sidebar';

export function NavProjects({
  projects,
}: {
  projects: {
    name: string;
    icon: LucideIcon;
    link: ComponentProps<typeof Link>;
  }[];
}) {
  return (
    <SidebarGroup class="group-data-[collapsible=icon]:hidden">
      <SidebarGroupLabel>Projects</SidebarGroupLabel>
      <SidebarMenu>
        <For each={projects}>
          {(item) => (
            <SidebarMenuItem>
              <SidebarMenuButton as={Link} {...item?.link}>
                <item.icon />
                <span>{item.name}</span>
              </SidebarMenuButton>
              <DropdownMenu>
                <DropdownMenuTrigger as={SidebarMenuAction} showOnHover>
                  <MoreHorizontal />
                  <span class="sr-only">More</span>
                </DropdownMenuTrigger>
                <DropdownMenuContent class="w-48 rounded-lg">
                  <DropdownMenuItem>
                    <Folder class="text-muted-foreground" />
                    <span>View Project</span>
                  </DropdownMenuItem>
                  <DropdownMenuItem>
                    <Forward class="text-muted-foreground" />
                    <span>Share Project</span>
                  </DropdownMenuItem>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem>
                    <Trash2 class="text-muted-foreground" />
                    <span>Delete Project</span>
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </SidebarMenuItem>
          )}
        </For>
        <SidebarMenuItem>
          <SidebarMenuButton class="text-sidebar-foreground/70">
            <MoreHorizontal class="text-sidebar-foreground/70" />
            <span>More</span>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarGroup>
  );
}
