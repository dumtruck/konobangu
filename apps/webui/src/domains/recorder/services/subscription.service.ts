import {
  SubscriptionCategoryEnum,
  type SubscriptionsInsertInput,
} from '@/infra/graphql/gql/graphql';
import { Injectable, inject } from '@outposts/injection-js';
import { omit } from 'lodash-es';
import { buildMikanSubscriptionSeasonSourceUrl } from '../schema/mikan';
import type { SubscriptionInsertForm } from '../schema/subscriptions';
import { MikanService } from './mikan.service';

@Injectable()
export class SubscriptionService {
  private mikan = inject(MikanService);

  transformInsertFormToInput(
    form: SubscriptionInsertForm
  ): SubscriptionsInsertInput {
    if (form.category === SubscriptionCategoryEnum.MikanSeason) {
      return {
        ...omit(form, ['seasonStr', 'year']),
        sourceUrl: buildMikanSubscriptionSeasonSourceUrl(
          this.mikan.mikanBaseUrl,
          form
        ).toString(),
      };
    }
    return form;
  }
}
