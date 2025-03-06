import { runInInjectionContext } from '@outposts/injection-js';
import { autoLoginPartialRoutesGuard } from 'oidc-client-rx';
import { firstValueFrom } from 'rxjs';
import type { RouterContext } from '~/traits/router';

export const beforeLoadGuard = async ({
  context,
}: { context: RouterContext }) => {
  if (!context.isAuthenticated()) {
    const guard$ = runInInjectionContext(context.injector, () =>
      autoLoginPartialRoutesGuard()
    );

    const isAuthenticated = await firstValueFrom(guard$);
    if (!isAuthenticated) {
      throw !isAuthenticated;
    }
  }
};
