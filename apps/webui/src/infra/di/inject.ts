import { type InjectionToken, Injector, inject } from '@outposts/injection-js';

export function injectInjector(): Injector {
  return inject(Injector as any as InjectionToken<Injector>);
}
