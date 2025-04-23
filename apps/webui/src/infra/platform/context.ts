import { DOCUMENT } from './injection';

export const providePlatform = () => {
  return [{ provide: DOCUMENT, useValue: document }];
};
