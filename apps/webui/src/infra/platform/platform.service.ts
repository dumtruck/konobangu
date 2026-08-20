import { inject } from "@securitydept/client";
import { PLATFORM_INFO } from "@/app/environment/traits";

export class PlatformService {
  private readonly info = inject(PLATFORM_INFO);

  /** Resolved once by the environment; never discovered per call. */
  get userAgent(): string {
    return this.info.userAgent;
  }
}
