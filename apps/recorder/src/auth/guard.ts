import type { ParsedLocation } from '@tanstack/react-router';
import type { RouterContext } from '../controllers/__root';
import { PostLoginRedirectUriKey } from './config';

export const beforeLoadGuard = async ({
  context,
  location,
  // biome-ignore lint/complexity/noBannedTypes: <explanation>
}: { context: RouterContext; location: ParsedLocation<{}> }) => {
  if (!context.isAuthenticated) {
    // TODO: FIXME
    const user = await context.userManager.getUser();
    if (!user) {
      try {
        sessionStorage.setItem(PostLoginRedirectUriKey, location.href);
        // biome-ignore lint/suspicious/noEmptyBlockStatements: <explanation>
      } catch {}
      throw await context.auth.signinRedirect();
    }
  }
};
