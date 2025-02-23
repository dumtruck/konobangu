import type { Observable } from '@graphiql/toolkit';
import { InjectionToken, inject } from '@outposts/injection-js';
import {
  type AuthFeature,
  EventTypes,
  PublicEventsService,
} from 'oidc-client-rx';
import { filter, shareReplay } from 'rxjs';

export type CheckAuthResultEventType =
  | { type: EventTypes.CheckingAuthFinished }
  | {
      type: EventTypes.CheckingAuthFinishedWithError;
      value: string;
    };
export const CHECK_AUTH_RESULT_EVENT = new InjectionToken<
  Observable<CheckAuthResultEventType>
>('CHECK_AUTH_RESULT_EVENT');

export function withCheckAuthResultEvent(): AuthFeature {
  return {
    ɵproviders: [
      {
        provide: CHECK_AUTH_RESULT_EVENT,
        useFactory: () => {
          const publishEventService = inject(PublicEventsService);

          return publishEventService.registerForEvents().pipe(
            filter(
              (e) =>
                e.type === EventTypes.CheckingAuthFinishedWithError ||
                e.type === EventTypes.CheckingAuthFinished
            ),
            shareReplay(1)
          );
        },
        deps: [PublicEventsService],
      },
    ],
  };
}
