import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from '@/components/ui/sidebar';

import { Image } from '@/components/ui/image';

export function AppIcon() {
  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <SidebarMenuButton
          size="lg"
          className="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
        >
          <div className="flex size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground">
            <div className="relative size-8">
              <Image
                src="/assets/favicon.png"
                alt="App Logo"
                className="object-cover"
              />
            </div>
          </div>
          <div className="grid flex-1 gap-1 py-1 text-left text-sm leading-tight">
            <span className="truncate font-semibold">Konobangu</span>
            <span className="mt-1 truncate">@dumtruck</span>
          </div>
        </SidebarMenuButton>
      </SidebarMenuItem>
    </SidebarMenu>
  );
}
