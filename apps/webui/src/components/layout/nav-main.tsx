'use client';

import { ChevronRight, type LucideIcon } from 'lucide-react';

import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '@/components/ui/collapsible';
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from '@/components/ui/sidebar';
import { useMatches } from '@tanstack/react-router';
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
    return matches.some((match) => match.pathname.startsWith(linkTo));
  };

  const renderSidebarMenuItemButton = (item: NavMainItem) => {
    return (
      <>
        {item.icon && <item.icon />}
        <span>{item.title}</span>
        <ChevronRight className="ml-auto transition-transform duration-200 group-data-[state=open]:rotate-90" />
      </>
    );
  };

  return groups.map((group, groupIndex) => {
    return (
      <SidebarGroup key={groupIndex}>
        <SidebarGroupLabel>{group.group}</SidebarGroupLabel>
        <SidebarMenu>
          {group.items.map((item, itemIndex) => {
            if (!item.children?.length) {
              return (
                <SidebarMenuItem key={itemIndex}>
                  <SidebarMenuButton asChild tooltip={item.title}>
                    <ProLink {...item.link}>
                      {renderSidebarMenuItemButton(item)}
                    </ProLink>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              );
            }
            return (
              <Collapsible
                key={itemIndex}
                asChild
                className="group/collapsible"
                defaultOpen={isMenuMatch(item.link)}
              >
                <SidebarMenuItem>
                  <CollapsibleTrigger asChild>
                    <SidebarMenuButton tooltip={item.title}>
                      {renderSidebarMenuItemButton(item)}
                    </SidebarMenuButton>
                  </CollapsibleTrigger>
                  <CollapsibleContent>
                    <SidebarMenuSub>
                      {(item.children || []).map((subItem, subItemIndex) => {
                        return (
                          <SidebarMenuSubItem key={subItemIndex}>
                            <SidebarMenuSubButton
                              asChild
                              isActive={isMenuMatch(subItem.link)}
                            >
                              <ProLink
                                {...subItem.link}
                                activeProps={{ className: '' }}
                              >
                                <span>{subItem.title}</span>
                              </ProLink>
                            </SidebarMenuSubButton>
                          </SidebarMenuSubItem>
                        );
                      })}
                    </SidebarMenuSub>
                  </CollapsibleContent>
                </SidebarMenuItem>
              </Collapsible>
            );
          })}
        </SidebarMenu>
      </SidebarGroup>
    );
  });
}
