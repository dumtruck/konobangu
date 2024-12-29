'use client';

import { Fragment, type ReactNode } from 'react';

export const analytics = {
  isFeatureEnabled: async (_key: string, _userId: string): Promise<boolean | null> => {
    return false;
  },
  capture: (_event: string, _properties: Record<string, unknown>): void => {

  },
  identify(_userId: string, _properties: Record<string, unknown>): void {

  }
}

type AnalyticsImplProviderProps = {
  readonly children: ReactNode;
};

export const AnalyticsImplProvider = (
  properties: Omit<AnalyticsImplProviderProps, 'client'>
) => <Fragment {...properties} />;
