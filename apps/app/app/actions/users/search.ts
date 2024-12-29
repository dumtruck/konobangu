'use server';

import {
  getFullOrganizationFromSession,
  getSessionFromHeaders,
} from '@konobangu/auth/server';
import Fuse from 'fuse.js';

export const searchUsers = async (
  query: string
): Promise<
  | {
      data: string[];
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

    const users = members.map((user) => ({
      id: user.id,
      name: user.user.name ?? user.user.email ?? 'Unknown user',
      imageUrl: user.user.image,
    }));

    const fuse = new Fuse(users, {
      keys: ['name'],
      minMatchCharLength: 1,
      threshold: 0.3,
    });

    const results = fuse.search(query);
    const data = results.map((result) => result.item.id);

    return { data };
  } catch (error) {
    return { error };
  }
};
