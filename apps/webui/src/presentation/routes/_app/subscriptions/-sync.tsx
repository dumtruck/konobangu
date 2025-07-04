import { Button } from '@/components/ui/button';
import {
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Spinner } from '@/components/ui/spinner';
import { INSERT_SUBSCRIBER_TASK } from '@/domains/recorder/schema/tasks';
import {
  type InsertSubscriberTaskMutation,
  type InsertSubscriberTaskMutationVariables,
  SubscriberTaskTypeEnum,
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
    const [insertSubscriberTask, { loading: loadingInsert }] = useMutation<
      InsertSubscriberTaskMutation,
      InsertSubscriberTaskMutationVariables
    >(INSERT_SUBSCRIBER_TASK, {
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

    const loading = loadingInsert;

    return (
      <div className="flex flex-col gap-2">
        <Button
          size="lg"
          variant="outline"
          onClick={() =>
            insertSubscriberTask({
              variables: {
                data: {
                  job: {
                    subscriptionId: id,
                    taskType: SubscriberTaskTypeEnum.SyncOneSubscriptionSources,
                  },
                },
              },
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
            insertSubscriberTask({
              variables: {
                data: {
                  job: {
                    subscriptionId: id,
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
            insertSubscriberTask({
              variables: {
                data: {
                  job: {
                    subscriptionId: id,
                    taskType:
                      SubscriberTaskTypeEnum.SyncOneSubscriptionFeedsFull,
                  },
                },
              },
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
  isCron?: boolean;
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
