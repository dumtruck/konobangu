import type { ReactNode } from 'react';
import { AnalyticsImplProvider } from './client';

type AnalyticsProviderProps = {
  readonly children: ReactNode;
};

export const AnalyticsProvider = ({ children }: AnalyticsProviderProps) => (
  <AnalyticsImplProvider>
    {children}
  </AnalyticsImplProvider>
);
