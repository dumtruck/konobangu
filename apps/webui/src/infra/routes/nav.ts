import type { ProLinkProps } from '@/components/ui/pro-link';
import { type } from 'arktype';
import {
  BookOpen,
  Folders,
  KeyRound,
  type LucideIcon,
  Settings2,
  SquareTerminal,
  Telescope,
} from 'lucide-react';

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

export const CreateCompleteAction = {
  Back: 'back',
  Detail: 'detail',
} as const;

export type CreateCompleteAction =
  (typeof CreateCompleteAction)[keyof typeof CreateCompleteAction];

export const CreateCompleteActionSchema = type.enumerated(
  ...Object.values(CreateCompleteAction)
);
