import { AppAuthMethod, AuthMethodEnum } from '@/app/auth/config';
import { useAuth } from '@/app/auth/hooks';
import { ProLink } from '@/components/ui/pro-link';
import { Spinner } from '@/components/ui/spinner';
import { createFileRoute } from '@tanstack/react-router';
import { useAtom } from 'jotai/react';
import { atomWithObservable } from 'jotai/utils';
import { EventTypes } from 'oidc-client-rx';
import { useMemo } from 'react';
import { filter, map } from 'rxjs';

export const Route = createFileRoute('/auth/oidc/callback')({
  component: OidcCallbackRouteComponent,
});

function OidcCallbackRouteComponent() {
  const { authContext } = useAuth();

  const isLoading = useAtom(
    useMemo(
      () =>
        atomWithObservable(() =>
          authContext.checkAuthResultEvent$.pipe(map(Boolean))
        ),
      [authContext.checkAuthResultEvent$]
    )
  );

  const checkAuthResultError = useAtom(
    useMemo(
      () =>
        atomWithObservable(() =>
          authContext.checkAuthResultEvent$.pipe(
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
        ),
      [authContext.checkAuthResultEvent$]
    )
  );

  const renderBackToHome = () => {
    return (
      <ProLink
        to="/"
        className="inline-flex h-10 items-center rounded-md border border-gray-200 bg-white px-8 font-medium text-sm shadow-sm transition-colors hover:bg-gray-100 hover:text-gray-900 dark:border-gray-800 dark:bg-gray-950 dark:focus-visible:ring-gray-300 dark:hover:bg-gray-800 dark:hover:text-gray-50"
      >
        Return to website
      </ProLink>
    );
  };

  return (
    <div className="flex h-svh items-center px-4 py-12 sm:px-6 md:px-8 lg:px-12 xl:px-16">
      <div className="w-full space-y-6 text-center">
        <div className="space-y-6">
          <div className="flex justify-center font-bold text-4xl tracking-tighter sm:text-5xl">
            <Spinner variant="circle-filled" size="48" />
          </div>
          {AppAuthMethod === AuthMethodEnum.OIDC ? (
            <>
              {isLoading && (
                <p className="text-gray-500">
                  Processing OIDC authentication callback...
                </p>
              )}
              {checkAuthResultError && (
                <p className="text-gray-500">
                  Failed to handle OIDC callback: {checkAuthResultError}
                </p>
              )}
              {!isLoading && !checkAuthResultError && (
                <p className="text-gray-500">
                  Succeed to handle OIDC authentication callback.
                </p>
              )}
            </>
          ) : (
            <p className="text-gray-500">
              Error: Current authentication method is not OIDC!
            </p>
          )}
          {renderBackToHome()}
        </div>
      </div>
    </div>
  );
}
