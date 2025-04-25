import type { RouterContext } from '@/infra/routes/traits';
import { firstValueFrom } from 'rxjs';
import { authContextFromInjector } from './context';

export const beforeLoadGuard = async ({
  context,
}: { context: RouterContext }) => {
  const { isAuthenticated$, authProvider } = authContextFromInjector(
    context.injector
  );
  if (!(await firstValueFrom(isAuthenticated$))) {
    const isAuthenticated = await firstValueFrom(
      authProvider.autoLoginPartialRoutesGuard()
    );
    if (!isAuthenticated) {
      throw !isAuthenticated;
    }
  }
};
