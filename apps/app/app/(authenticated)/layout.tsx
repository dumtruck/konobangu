import { getSessionFromHeaders } from '@konobangu/auth/server';
import { SidebarProvider } from '@konobangu/design-system/components/ui/sidebar';
import { env } from '@konobangu/env';
import { showBetaFeature } from '@konobangu/feature-flags';
import { secure } from '@konobangu/security';
import { redirect } from 'next/navigation';
import type { ReactNode } from 'react';
import { PostHogIdentifier } from './components/posthog-identifier';
import { GlobalSidebar } from './components/sidebar';

type AppLayoutProperties = {
  readonly children: ReactNode;
};

const AppLayout = async ({ children }: AppLayoutProperties) => {
  if (env.ARCJET_KEY) {
    await secure(['CATEGORY:PREVIEW']);
  }

  const { user } = await getSessionFromHeaders();

  if (!user) {
    return redirect('/sign-in'); // from next/navigation
  }
  const betaFeature = await showBetaFeature();

  return (
    <SidebarProvider>
      <GlobalSidebar>
        {betaFeature && (
          <div className="m-4 rounded-full bg-success p-1.5 text-center text-sm text-success-foreground">
            Beta feature now available
          </div>
        )}
        {children}
      </GlobalSidebar>
      <PostHogIdentifier />
    </SidebarProvider>
  );
};

export default AppLayout;
