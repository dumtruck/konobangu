import { AUTH_PROVIDER } from '@/infra/auth/auth.provider';
import { Injectable, inject } from '@outposts/injection-js';

@Injectable()
export class AuthService {
  private authProvider = inject(AUTH_PROVIDER);

  isAuthenticated$ = this.authProvider.isAuthenticated$;
  checkAuthResultEvent$ = this.authProvider.checkAuthResultEvent$;
  authData$ = this.authProvider.authData$;

  setup() {
    this.authProvider.setup();
  }
}
