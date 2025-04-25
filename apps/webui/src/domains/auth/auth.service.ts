import { AUTH_PROVIDER } from '@/infra/auth/auth.provider';
import { Injectable, inject } from '@outposts/injection-js';

@Injectable()
export class AuthService {
  private authProvider = inject(AUTH_PROVIDER);

  isAuthenticated$ = this.authProvider.isAuthenticated$;
  userData$ = this.authProvider.userData$;
  checkAuthResultEvent$ = this.authProvider.checkAuthResultEvent$;

  setup() {
    this.authProvider.setup();
  }
}
