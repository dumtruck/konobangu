import { inject } from '@outposts/injection-js';
import { DOCUMENT } from '../platform/injection';

export class IntlService {
  document = inject(DOCUMENT);

  formatTimestamp(timestamp: number, options?: Intl.DateTimeFormatOptions) {
    const defaultOptions: Intl.DateTimeFormatOptions = {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      hour12: false,
      ...options,
    };

    return new Intl.DateTimeFormat(
      this.document.defaultView?.navigator.language,
      {
        ...defaultOptions,
        ...options,
      }
    ).format(new Date(timestamp));
  }
}
