import { AuthService } from '@/domains/auth/auth.service';
import { Injectable, inject } from '@outposts/injection-js';

@Injectable()
export class SubscriberService {
  authService = inject(AuthService);
}
