import { Outlet, createRootRoute } from '@tanstack/solid-router';
// import {
//   NavigationMenu,
//   NavigationMenuItem,
//   NavigationMenuLink,
//   NavigationMenuTrigger,
// } from '~/components/ui/navigation-menu';

export const Route = createRootRoute({
  component: () => {
    return (
      <>
        {/* <div class="sticky inset-x-0 top-0 isolate z-10 flex shrink-0 items-center gap-2 border-b bg-background">
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
        </div> */}
        <Outlet />
      </>
    );
  },
});
