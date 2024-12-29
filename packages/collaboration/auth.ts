import 'server-only';
import { env } from '@konobangu/env';
import { Liveblocks as LiveblocksNode } from '@liveblocks/node';

type AuthenticateOptions = {
  userId: string;
  orgId: string;
  userInfo: Liveblocks['UserMeta']['info'];
};

export const authenticate = async ({
  userId,
  orgId,
  userInfo,
}: AuthenticateOptions) => {
  const liveblocks = new LiveblocksNode({
    secret: env.LIVEBLOCKS_SECRET,
  });

  // Start an auth session inside your endpoint
  const session = liveblocks.prepareSession(userId, { userInfo });

  // Use a naming pattern to allow access to rooms with wildcards
  // Giving the user write access on their organization
  session.allow(`${orgId}:*`, session.FULL_ACCESS);

  // Authorize the user and return the result
  const { status, body } = await session.authorize();

  return new Response(body, { status });
};
