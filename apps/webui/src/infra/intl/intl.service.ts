import { inject } from "@securitydept/client";
import { LOCALE } from "@/app/environment/traits";
import { formatInstant, parseGraphqlDatetime } from "../time/instant";

export { parseGraphqlDatetime } from "../time/instant";

export class IntlService {
  /** Resolved and non-empty; the environment guarantees a preferred locale. */
  private readonly locale = inject(LOCALE);

  get timezone() {
    return Temporal.Now.timeZoneId();
  }

  formatTimestamp(timestamp: number, options?: Intl.DateTimeFormatOptions) {
    return formatInstant(
      Temporal.Instant.fromEpochMilliseconds(timestamp),
      options,
      this.locale,
    );
  }

  formatDatetimeWithTz(datetime: string, options?: Intl.DateTimeFormatOptions) {
    try {
      return formatInstant(
        parseGraphqlDatetime(datetime),
        { timeZoneName: "short", ...options },
        this.locale,
      );
    } catch {
      return "Invalid timestamp";
    }
  }
}
