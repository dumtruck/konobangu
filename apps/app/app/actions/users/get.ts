'use server';

import {
  getFullOrganizationFromSession,
  getSessionFromHeaders,
} from '@konobangu/auth/server';
import { tailwind } from '@konobangu/tailwind-config';

const colors = [
  tailwind.theme.colors.red[500],
  tailwind.theme.colors.orange[500],
  tailwind.theme.colors.amber[500],
  tailwind.theme.colors.yellow[500],
  tailwind.theme.colors.lime[500],
  tailwind.theme.colors.green[500],
  tailwind.theme.colors.emerald[500],
  tailwind.theme.colors.teal[500],
  tailwind.theme.colors.cyan[500],
  tailwind.theme.colors.sky[500],
  tailwind.theme.colors.blue[500],
  tailwind.theme.colors.indigo[500],
  tailwind.theme.colors.violet[500],
  tailwind.theme.colors.purple[500],
  tailwind.theme.colors.fuchsia[500],
  tailwind.theme.colors.pink[500],
  tailwind.theme.colors.rose[500],
];

export const getUsers = async (
  userIds: string[]
): Promise<
  | {
      data: Liveblocks['UserMeta']['info'][];
    }
  | {
      error: unknown;
    }
> => {
  try {
    const session = await getSessionFromHeaders();
    const { orgId } = session;

    if (!orgId) {
      throw new Error('Not logged in');
    }

    const { fullOrganization } = await getFullOrganizationFromSession(session);

    const members = fullOrganization?.members || [];

    const data: Liveblocks['UserMeta']['info'][] = members
      .filter((user) => user?.userId && userIds.includes(user?.userId))
      .map((user) => ({
        name: user.user.name ?? user.user.email ?? 'Unknown user',
        picture: user.user.image,
        color: colors[Math.floor(Math.random() * colors.length)],
      }));

    return { data };
  } catch (error) {
    return { error };
  }
};
