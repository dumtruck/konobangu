import { Image } from '@kobalte/core/image';
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from '~/components/ui/sidebar';

export function AppIcon() {
  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <SidebarMenuButton
          size="lg"
          class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
        >
          <div class="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground">
            <Image fallbackDelay={1000}>
              <Image.Img
                src="/assets/favicon.png"
                class="size-8 object-cover"
              />
              <Image.Fallback>KONOBANGU</Image.Fallback>
            </Image>
          </div>
          <div class="grid flex-1 gap-1 py-1 text-left text-sm leading-tight">
            <span class="truncate font-semibold">Konobangu</span>
            <span class="mt-1 truncate">@dumtruck</span>
          </div>
        </SidebarMenuButton>
      </SidebarMenuItem>
    </SidebarMenu>
  );
}
