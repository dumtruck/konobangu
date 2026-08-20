import {
  RouterNavigationIntent,
  RouterNavigationMode,
  UriReferenceString,
} from "@securitydept/client";
import { ChevronsUpDown, LogOut } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";
import { useAuth } from "@/app/auth/provider";
import { APP_ROUTER } from "@/app/environment/traits";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from "@/components/ui/sidebar";
import { useInject } from "@/infra/di/inject";

export function NavUser() {
  const { isMobile } = useSidebar();
  const router = useInject(APP_ROUTER);
  const { authData: principal, authService } = useAuth();
  const [pending, setPending] = useState(false);
  const name = principal?.displayName ?? principal?.subject ?? "Signed out";
  async function logout() {
    setPending(true);
    try {
      await authService.logout();
      // Stay on a public page until the user deliberately starts another login.
      await router.navigate({
        url: UriReferenceString.parse("/auth/signed-out"),
        mode: RouterNavigationMode.Replace,
        intent: RouterNavigationIntent.PostAuthRedirect,
      });
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Sign out failed");
    } finally {
      setPending(false);
    }
  }
  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <DropdownMenu>
          <DropdownMenuTrigger asChild aria-label="User menu">
            <SidebarMenuButton size="lg">
              <Avatar className="h-8 w-8 rounded-lg">
                <AvatarImage alt={name} />
                <AvatarFallback>{name.slice(0, 2)}</AvatarFallback>
              </Avatar>
              <div className="grid flex-1 text-left text-xs">
                <span className="truncate font-semibold">{name}</span>
                <span className="truncate">{principal?.issuer}</span>
              </div>
              <ChevronsUpDown className="ml-auto size-4" />
            </SidebarMenuButton>
          </DropdownMenuTrigger>
          <DropdownMenuContent side={isMobile ? "bottom" : "right"} align="end">
            <DropdownMenuLabel>{name}</DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuItem
              disabled={pending}
              onSelect={() => {
                void logout();
              }}
            >
              <LogOut />
              Sign out
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  );
}
