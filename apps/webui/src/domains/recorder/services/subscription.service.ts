import {
  SubscriptionCategoryEnum,
  type SubscriptionsInsertInput,
} from '@/infra/graphql/gql/graphql';
import { Injectable, inject } from '@outposts/injection-js';
import { ArkErrors } from 'arktype';
import { omit } from 'lodash-es';
import {
  type MikanSubscriptionBangumiSourceUrl,
  type MikanSubscriptionSeasonSourceUrl,
  type MikanSubscriptionSubscriberSourceUrl,
  buildMikanSubscriptionSeasonSourceUrl,
  extractMikanSubscriptionBangumiSourceUrl,
  extractMikanSubscriptionSeasonSourceUrl,
  extractMikanSubscriptionSubscriberSourceUrl,
} from '../schema/mikan';
import type { SubscriptionForm } from '../schema/subscriptions';
import { MikanService } from './mikan.service';

@Injectable()
export class SubscriptionService {
  private mikan = inject(MikanService);

  transformInsertFormToInput(form: SubscriptionForm): SubscriptionsInsertInput {
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

  extractSourceUrlMeta(
    category: SubscriptionCategoryEnum,
    sourceUrl: string
  ):
    | MikanSubscriptionSeasonSourceUrl
    | MikanSubscriptionBangumiSourceUrl
    | MikanSubscriptionSubscriberSourceUrl
    | null {
    let meta:
      | MikanSubscriptionSeasonSourceUrl
      | MikanSubscriptionBangumiSourceUrl
      | MikanSubscriptionSubscriberSourceUrl
      | null
      | ArkErrors = null;
    if (category === SubscriptionCategoryEnum.MikanSeason) {
      meta = extractMikanSubscriptionSeasonSourceUrl(sourceUrl);
    } else if (category === SubscriptionCategoryEnum.MikanBangumi) {
      meta = extractMikanSubscriptionBangumiSourceUrl(sourceUrl);
    } else if (category === SubscriptionCategoryEnum.MikanSubscriber) {
      meta = extractMikanSubscriptionSubscriberSourceUrl(sourceUrl);
    }
    return meta instanceof ArkErrors ? null : meta;
  }
}
