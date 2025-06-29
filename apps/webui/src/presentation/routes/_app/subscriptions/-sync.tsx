import { Button } from '@/components/ui/button';
import {
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Spinner } from '@/components/ui/spinner';
import {
  SYNC_SUBSCRIPTION_FEEDS_FULL,
  SYNC_SUBSCRIPTION_FEEDS_INCREMENTAL,
  SYNC_SUBSCRIPTION_SOURCES,
} from '@/domains/recorder/schema/subscriptions';
import {
  SubscriberTaskTypeEnum,
  type SyncSubscriptionFeedsFullMutation,
  type SyncSubscriptionFeedsFullMutationVariables,
  type SyncSubscriptionFeedsIncrementalMutation,
  type SyncSubscriptionFeedsIncrementalMutationVariables,
  type SyncSubscriptionSourcesMutation,
  type SyncSubscriptionSourcesMutationVariables,
} from '@/infra/graphql/gql/graphql';
import { useMutation } from '@apollo/client';
import { useNavigate } from '@tanstack/react-router';
import { RefreshCcwIcon } from 'lucide-react';
import { memo, useCallback } from 'react';
import { toast } from 'sonner';

export type SubscriptionSyncViewCompletePayload = {
  id: string;
};

export interface SubscriptionSyncViewProps {
  id: number;
  onComplete: (payload: SubscriptionSyncViewCompletePayload) => void;
}

export const SubscriptionSyncView = memo(
  ({ id, onComplete }: SubscriptionSyncViewProps) => {
    const [syncSubscriptionFeedsIncremental, { loading: loadingIncremental }] =
      useMutation<
        SyncSubscriptionFeedsIncrementalMutation,
        SyncSubscriptionFeedsIncrementalMutationVariables
      >(SYNC_SUBSCRIPTION_FEEDS_INCREMENTAL, {
        onCompleted: (data) => {
          toast.success('Sync completed');
          onComplete(data.subscriberTasksCreateOne);
        },
        onError: (error) => {
          toast.error('Failed to sync subscription', {
            description: error.message,
          });
        },
      });

    const [syncSubscriptionFeedsFull, { loading: loadingFull }] = useMutation<
      SyncSubscriptionFeedsFullMutation,
      SyncSubscriptionFeedsFullMutationVariables
    >(SYNC_SUBSCRIPTION_FEEDS_FULL, {
      onCompleted: (data) => {
        toast.success('Sync completed');
        onComplete(data.subscriptionsSyncOneFeedsFull);
      },
      onError: (error) => {
        toast.error('Failed to sync subscription', {
          description: error.message,
        });
      },
    });

    const [syncSubscriptionSources, { loading: loadingSources }] = useMutation<
      SyncSubscriptionSourcesMutation,
      SyncSubscriptionSourcesMutationVariables
    >(SYNC_SUBSCRIPTION_SOURCES, {
      onCompleted: (data) => {
        toast.success('Sync completed');
        onComplete(data.subscriptionsSyncOneSources);
      },
      onError: (error) => {
        toast.error('Failed to sync subscription', {
          description: error.message,
        });
      },
    });

    const loading = loadingIncremental || loadingFull || loadingSources;

    return (
      <div className="flex flex-col gap-2">
        <Button
          size="lg"
          variant="outline"
          onClick={() =>
            syncSubscriptionSources({
              variables: { filter: { id: { eq: id } } },
            })
          }
        >
          <RefreshCcwIcon className="h-4 w-4" />
          <span>Sources</span>
        </Button>
        <Button
          size="lg"
          variant="outline"
          onClick={() =>
            syncSubscriptionFeedsIncremental({
              variables: {
                data: {
                  job: {
                    subscriberId: id,
                    taskType:
                      SubscriberTaskTypeEnum.SyncOneSubscriptionFeedsIncremental,
                  },
                },
              },
            })
          }
        >
          <RefreshCcwIcon className="h-4 w-4" />
          <span>Incremental Feeds</span>
        </Button>
        <Button
          size="lg"
          variant="outline"
          onClick={() =>
            syncSubscriptionFeedsFull({
              variables: { filter: { id: { eq: id } } },
            })
          }
        >
          <RefreshCcwIcon className="h-4 w-4" />
          <span>Full Feeds</span>
        </Button>

        {loading && (
          <div className="absolute inset-0 flex flex-row items-center justify-center gap-2">
            <Spinner variant="circle-filled" size="16" />
            <span>Syncing...</span>
          </div>
        )}
      </div>
    );
  }
);

export interface SubscriptionSyncDialogContentProps {
  id: number;
  onCancel?: VoidFunction;
}

export const SubscriptionSyncDialogContent = memo(
  ({ id, onCancel }: SubscriptionSyncDialogContentProps) => {
    const navigate = useNavigate();

    const handleSyncComplete = useCallback(
      (payload: SubscriptionSyncViewCompletePayload) => {
        navigate({
          to: '/tasks/detail/$id',
          params: {
            id: `${payload.id}`,
          },
        });
      },
      [navigate]
    );

    return (
      <DialogContent onAbort={onCancel}>
        <DialogHeader>
          <DialogTitle>Sync Subscription</DialogTitle>
          <DialogDescription>
            Sync the subscription with sources and feeds.
          </DialogDescription>
        </DialogHeader>
        <SubscriptionSyncView id={id} onComplete={handleSyncComplete} />
      </DialogContent>
    );
  }
);
