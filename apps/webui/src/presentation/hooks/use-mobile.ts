import { useInject } from '@/infra/di/inject';
import { ThemeService } from '@/infra/styles/theme.service';
import { useAtomValue } from 'jotai/react';
import { atomWithObservable } from 'jotai/utils';
import { useMemo } from 'react';

export function useIsMobile() {
  const themeService = useInject(ThemeService);

  const isMobile = useAtomValue(
    useMemo(
      () =>
        atomWithObservable(() => themeService.isMobile$, {
          initialValue: themeService.isMobile$.value,
        }),
      [themeService.isMobile$]
    )
  );

  return isMobile;
}
