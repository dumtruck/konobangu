import { ChevronRight, type LucideIcon } from 'lucide-solid';
import { For, Show, createSignal } from 'solid-js';

import { useMatch, useMatches } from '@tanstack/solid-router';
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '~/components/ui/collapsible';
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from '~/components/ui/sidebar';
import { ProLink, type ProLinkProps } from '../ui/pro-link';

export interface NavMainItem {
  link?: ProLinkProps;
  title: string;
  icon?: LucideIcon;
  children?: { title: string; link: ProLinkProps }[];
}

export interface NavMainGroup {
  group: string;
  items: NavMainItem[];
}

export function NavMain({
  groups,
}: {
  groups: NavMainGroup[];
}) {
  const matches = useMatches();

  const isMenuMatch = (link: ProLinkProps | undefined) => {
    const linkTo = link?.to;
    if (!linkTo) {
      return false;
    }
    return matches().some((match) => match.pathname.startsWith(linkTo));
  };

  const renderSidebarMenuItemButton = (item: NavMainItem) => {
    return (
      <>
        {item.icon && <item.icon />}
        <span>{item.title}</span>
        <ChevronRight class="ml-auto transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90" />
      </>
    );
  };

  return (
    <For each={groups}>
      {(group) => (
        <SidebarGroup>
          <SidebarGroupLabel>{group.group}</SidebarGroupLabel>
          <SidebarMenu>
            <For each={group.items}>
              {(item) => {
                return (
                  <Show
                    when={!!item.children?.length}
                    fallback={
                      <SidebarMenuItem>
                        <SidebarMenuButton
                          as={ProLink}
                          {...item.link}
                          tooltip={item.title}
                        >
                          {renderSidebarMenuItemButton(item)}
                        </SidebarMenuButton>
                      </SidebarMenuItem>
                    }
                  >
                    <Collapsible
                      as={SidebarMenuItem}
                      class="group/collapsible"
                      defaultOpen={isMenuMatch(item.link)}
                    >
                      <CollapsibleTrigger
                        as={SidebarMenuButton}
                        tooltip={item.title}
                      >
                        {renderSidebarMenuItemButton(item)}
                      </CollapsibleTrigger>
                      <CollapsibleContent>
                        <SidebarMenuSub>
                          <For each={item.children || []}>
                            {(subItem) => (
                              <SidebarMenuSubItem>
                                <SidebarMenuSubButton
                                  as={ProLink}
                                  {...subItem.link}
                                  isActive={isMenuMatch(subItem.link)}
                                  activeProps={{ class: '' }}
                                >
                                  <span>{subItem.title}</span>
                                </SidebarMenuSubButton>
                              </SidebarMenuSubItem>
                            )}
                          </For>
                        </SidebarMenuSub>
                      </CollapsibleContent>
                    </Collapsible>
                  </Show>
                );
              }}
            </For>
          </SidebarMenu>
        </SidebarGroup>
      )}
    </For>
  );
}
