import type { Injector } from '@outposts/injection-js';
import { IntlService } from './intl.service';

export function intlContextFromInjector(injector: Injector) {
  const intlService = injector.get(IntlService);

  return {
    intlService,
  };
}
