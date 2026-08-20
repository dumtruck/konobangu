import { type CancellationTokenTrait, inject } from "@securitydept/client";
import { type } from "arktype";
import { HTTP_CLIENT } from "@/infra/http/http-client";

const AvailabilitySchema = type({ available: "boolean" });
export class Credential3rdService {
  private readonly http = inject(HTTP_CLIENT);
  checkAvailable(id: number, cancellationToken?: CancellationTokenTrait) {
    return this.http.json({
      endpoint: `credential3rd/${id}/check-available`,
      method: "POST",
      json: {},
      schema: AvailabilitySchema,
      cancellationToken,
      replay: "never",
    });
  }
}
