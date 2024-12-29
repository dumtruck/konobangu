import 'server-only';

export const analytics = {
  isFeatureEnabled: async (_key: string, _userId: string): Promise<boolean | null> => {
    return false;
  },
  capture: (_event: string, _properties: Record<string, unknown>): void => {

  },
  identify(_userId: string, _properties: Record<string, unknown>): void {

  }
}