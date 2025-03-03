import { Link, Outlet, createRootRoute } from '@tanstack/solid-router';
import {
  NavigationMenu,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuTrigger,
} from '~/components/ui/navigation-menu';

export const Route = createRootRoute({
  component: () => {
    return (
      <>
        <div class="flex space-y-4">
          <NavigationMenu orientation="horizontal">
            <NavigationMenuItem>
              <NavigationMenuTrigger>
                <NavigationMenuLink as={Link} to="/">
                  Home
                </NavigationMenuLink>
              </NavigationMenuTrigger>
            </NavigationMenuItem>
            <NavigationMenuItem>
              <NavigationMenuLink as={Link} to="/about">
                About
              </NavigationMenuLink>
            </NavigationMenuItem>
          </NavigationMenu>
        </div>
        <hr />
        <Outlet />
      </>
    );
  },
});
