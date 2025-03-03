import { ChevronsUpDown, type LucideIcon, Plus } from 'lucide-solid';
import { For, createSignal } from 'solid-js';
import { Dynamic } from 'solid-js/web';

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuTrigger,
} from '~/components/ui/dropdown-menu';
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from '~/components/ui/sidebar';

export function TeamSwitcher(props: {
  teams: {
    name: string;
    logo: LucideIcon;
    plan: string;
  }[];
}) {
  const [activeTeam, setActiveTeam] = createSignal(props.teams[0]);

  const logo = activeTeam().logo;

  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <DropdownMenu>
          <DropdownMenuTrigger
            as={SidebarMenuButton}
            size="lg"
            class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
          >
            <div class="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground">
              <Dynamic component={logo} class="size-4" />
            </div>
            <div class="grid flex-1 text-left text-sm leading-tight">
              <span class="truncate font-semibold">{activeTeam().name}</span>
              <span class="truncate text-xs">{activeTeam().plan}</span>
            </div>
            <ChevronsUpDown class="ml-auto" />
          </DropdownMenuTrigger>
          <DropdownMenuContent class="w-[--radix-dropdown-menu-trigger-width] min-w-56 rounded-lg">
            <DropdownMenuLabel class="text-muted-foreground text-xs">
              Teams
            </DropdownMenuLabel>
            <For each={props.teams}>
              {(team, index) => (
                <DropdownMenuItem
                  onClick={() => setActiveTeam(team)}
                  class="gap-2 p-2"
                >
                  <div class="flex size-6 items-center justify-center rounded-sm border">
                    <Dynamic component={team.logo} class="size-4 shrink-0" />
                  </div>
                  {team.name}
                  <DropdownMenuShortcut>⌘{index() + 1}</DropdownMenuShortcut>
                </DropdownMenuItem>
              )}
            </For>
            <DropdownMenuSeparator />
            <DropdownMenuItem class="gap-2 p-2">
              <div class="flex size-6 items-center justify-center rounded-md border bg-background">
                <Plus class="size-4" />
              </div>
              <div class="font-medium text-muted-foreground">Add team</div>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  );
}
