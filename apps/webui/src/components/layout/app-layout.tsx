import { useMatches } from '@tanstack/solid-router';
import {
  type ComponentProps,
  type FlowProps,
  For,
  Show,
  createMemo,
  splitProps,
} from 'solid-js';
import { Dynamic } from 'solid-js/web';
import { AppSidebar } from '~/components/layout/app-sidebar';
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbSeparator,
} from '~/components/ui/breadcrumb';
import { Separator } from '~/components/ui/separator';
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from '~/components/ui/sidebar';
import type { RouteStateDataOption } from '~/traits/router';
import type { RouteBreadcrumbItem } from '~/traits/router';
import { cn } from '~/utils/styles';
import { ProLink } from '../ui/pro-link';

export type AppAsideBreadcrumbItem = RouteBreadcrumbItem;
export interface AppAsideProps extends FlowProps<ComponentProps<'div'>> {
  breadcrumb?: AppAsideBreadcrumbItem[];
  extractBreadcrumbFromRoutes?: boolean;
}

export function AppAside(props: AppAsideProps) {
  const [local, other] = splitProps(props, [
    'children',
    'class',
    'breadcrumb',
    'extractBreadcrumbFromRoutes',
  ]);

  const matches = useMatches();

  const breadcrumb = createMemo(() => {
    if (local.breadcrumb) {
      return local.breadcrumb;
    }
    if (local.extractBreadcrumbFromRoutes) {
      return matches()
        .map((m, i, arr) => {
          const staticData = m.staticData as RouteStateDataOption;
          if (staticData.breadcrumb) {
            return {
              link:
                i + 1 >= arr.length
                  ? undefined
                  : {
                      to: m.pathname,
                    },
              ...staticData.breadcrumb,
            } as AppAsideBreadcrumbItem;
          }
          return undefined;
        })
        .filter((b): b is AppAsideBreadcrumbItem => !!b);
    }
    return [];
  });

  const breadcrumbLength = breadcrumb().length;

  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <header class="flex h-16 shrink-0 items-center gap-2 transition-[width,height] ease-linear group-has-[[data-collapsible=icon]]/sidebar-wrapper:h-12">
          <div class="flex items-center gap-2 px-4">
            <SidebarTrigger class="-ml-1" />
            <Show when={breadcrumbLength}>
              <Separator orientation="vertical" class="mr-2 h-4" />
              <Breadcrumb>
                <BreadcrumbList>
                  <For each={breadcrumb()}>
                    {(item, index) => {
                      const iconEl = (
                        <Show when={!!item.icon}>
                          <Dynamic component={item.icon} class=" size-4" />
                        </Show>
                      );

                      const isCurrent = index() + 1 === breadcrumbLength;

                      return (
                        <>
                          {index() > 0 && (
                            <BreadcrumbSeparator class="hidden md:block" />
                          )}
                          <BreadcrumbItem class="hidden md:block">
                            <BreadcrumbLink
                              class="text-[var(--foreground)] hover:text-inherit"
                              as={item.link ? ProLink : undefined}
                              current={isCurrent}
                              {...item?.link}
                            >
                              {iconEl}
                              {item.label}
                            </BreadcrumbLink>
                          </BreadcrumbItem>
                        </>
                      );
                    }}
                  </For>
                </BreadcrumbList>
              </Breadcrumb>
            </Show>
          </div>
        </header>
        <div {...other} class={cn('min-h-0 flex-1 p-4 pt-0', local.class)}>
          {local.children}
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}
