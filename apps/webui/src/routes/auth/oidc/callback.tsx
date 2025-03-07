import { createFileRoute } from '@tanstack/solid-router';
import { EventTypes } from 'oidc-client-rx';
import { filter, map } from 'rxjs';
import { Match, Switch, from } from 'solid-js';
import { isOidcAuth } from '~/auth/config';
import { useAuth } from '~/auth/hooks';
import { ProLink } from '~/components/ui/pro-link';
import { Spinner } from '~/components/ui/spinner';

export const Route = createFileRoute('/auth/oidc/callback')({
  component: OidcCallbackRouteComponent,
});

function OidcCallbackRouteComponent() {
  const auth = useAuth();

  const isLoading = from(auth.checkAuthResultEvent$.pipe(map(Boolean)));

  const checkAuthResultError = from(
    auth.checkAuthResultEvent$.pipe(
      filter(
        (
          e
        ): e is {
          type: EventTypes.CheckingAuthFinishedWithError;
          value: string;
        } => e.type === EventTypes.CheckingAuthFinishedWithError
      ),
      map((e) => e.value)
    )
  );

  const renderBackToHome = () => {
    return (
      <ProLink
        to="/"
        class="inline-flex h-10 items-center rounded-md border border-gray-200 border-gray-200 bg-white px-8 font-medium text-sm shadow-sm transition-colors hover:bg-gray-100 hover:text-gray-900 dark:border-gray-800 dark:border-gray-800 dark:bg-gray-950 dark:focus-visible:ring-gray-300 dark:hover:bg-gray-800 dark:hover:text-gray-50"
      >
        Return to website
      </ProLink>
    );
  };

  return (
    <div class="flex h-svh items-center px-4 py-12 sm:px-6 md:px-8 lg:px-12 xl:px-16">
      <div class="w-full space-y-6 text-center">
        <div class="space-y-6">
          <div class="flex justify-center font-bold text-4xl tracking-tighter sm:text-5xl">
            <Spinner variant="circle-filled" size="48" />
          </div>
          {isOidcAuth ? (
            <Switch
              fallback={
                <p class="text-gray-500">
                  Succeed to handle OIDC authentication callback.
                </p>
              }
            >
              <Match when={isLoading()}>
                <p class="text-gray-500">
                  Processing OIDC authentication callback...
                </p>
              </Match>
              <Match when={!!checkAuthResultError()}>
                <p class="text-gray-500">
                  Failed to handle OIDC callback: {checkAuthResultError()}
                </p>
              </Match>
            </Switch>
          ) : (
            <p class="text-gray-500">
              Error: Current authentication method is not OIDC!
            </p>
          )}
          {renderBackToHome()}
        </div>
      </div>
    </div>
  );
}
