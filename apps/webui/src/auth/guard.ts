import type { RouterContext } from '@/traits/router';
import { runInInjectionContext } from '@outposts/injection-js';
import { autoLoginPartialRoutesGuard } from 'oidc-client-rx';
import { firstValueFrom } from 'rxjs';
import { authContextFromInjector } from './context';

export const beforeLoadGuard = async ({
  context,
}: { context: RouterContext }) => {
  const { isAuthenticated$ } = authContextFromInjector(context.injector);
  if (!(await firstValueFrom(isAuthenticated$))) {
    const guard$ = runInInjectionContext(context.injector, () =>
      autoLoginPartialRoutesGuard()
    );

    const isAuthenticated = await firstValueFrom(guard$);
    if (!isAuthenticated) {
      throw !isAuthenticated;
    }
  }
};
