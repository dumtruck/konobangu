import { Injectable, inject } from '@outposts/injection-js';
import { OidcSecurityService } from 'oidc-client-rx';

@Injectable()
export class HttpService {
  authService = inject(OidcSecurityService);
}
