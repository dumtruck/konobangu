import {
  SubscriptionCategoryEnum,
  type SubscriptionsInsertInput,
} from '@/infra/graphql/gql/graphql';
import { Injectable, inject } from '@outposts/injection-js';
import { buildMikanSubscriptionSeasonSourceUrl } from '../schema/mikan';
import type { SubscriptionInsertForm } from '../schema/subscriptions';
import { MikanService } from './mikan.service';

@Injectable()
export class SubscriptionService {
  private mikan = inject(MikanService);

  transformInsertFormToInput(
    form: SubscriptionInsertForm
  ): SubscriptionsInsertInput {
    let sourceUrl: string;
    if (form.category === SubscriptionCategoryEnum.MikanSeason) {
      sourceUrl = buildMikanSubscriptionSeasonSourceUrl(
        this.mikan.mikanBaseUrl,
        form
      ).toString();
    } else {
      sourceUrl = form.sourceUrl;
    }
    return {
      ...form,
      sourceUrl,
    };
  }
}
