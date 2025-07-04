import { Cron } from '@/components/domains/cron';
import { CronMode } from '@/components/domains/cron/types';
import {
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Spinner } from '@/components/ui/spinner';
import { INSERT_CRON } from '@/domains/recorder/schema/cron';
import type {
  InsertCronMutation,
  InsertCronMutationVariables,
} from '@/infra/graphql/gql/graphql';
import { useMutation } from '@apollo/client';
import { useNavigate } from '@tanstack/react-router';
import { memo, useCallback } from 'react';
import { toast } from 'sonner';

const SUBSCRIPTION_TASK_CRON_PRESETS = [
  {
    label: 'Every hour',
    value: '0 0 * * * *',
    description: 'Runs at the top of every hour',
    category: 'common',
  },
  {
    label: 'Daily at midnight',
    value: '0 0 0 * * *',
    description: 'Runs once daily at 00:00',
    category: 'daily',
  },
  {
    label: 'Daily at 9 AM',
    value: '0 0 9 * * *',
    description: 'Runs daily at 9:00 AM',
    category: 'daily',
  },
  {
    label: 'Every Sunday',
    value: '0 0 0 * * 0',
    description: 'Runs every Sunday at midnight',
    category: 'weekly',
  },
  {
    label: 'First day of month',
    value: '0 0 0 1 * *',
    description: 'Runs on the 1st day of every month',
    category: 'monthly',
  },
  {
    label: 'Every year',
    value: '0 0 0 1 1 *',
    description: 'Runs on January 1st every year',
    category: 'yearly',
  },
];

export type SubscriptionCronCreationViewCompletePayload = {
  id: number;
};

export interface SubscriptionCronCreationViewProps {
  subscriptionId: number;
  onComplete: (payload: SubscriptionCronCreationViewCompletePayload) => void;
}

export const SubscriptionCronCreationView = memo(
  ({ subscriptionId, onComplete }: SubscriptionCronCreationViewProps) => {
    const [insertCron, { loading: loadingInsert }] = useMutation<
      InsertCronMutation,
      InsertCronMutationVariables
    >(INSERT_CRON, {
      onCompleted: (data) => {
        toast.success('Cron created');
        onComplete(data.cronCreateOne);
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
        <Cron
          mode={CronMode.Both}
          withCard={true}
          showPresets={false}
          presets={SUBSCRIPTION_TASK_CRON_PRESETS}
          timezone={'Asia/Shanghai'}
        />

        {loading && (
          <div className="absolute inset-0 flex flex-row items-center justify-center gap-2">
            <Spinner variant="circle-filled" size="16" />
            <span>Creating cron...</span>
          </div>
        )}
      </div>
    );
  }
);

export interface SubscriptionCronCreationDialogContentProps {
  subscriptionId: number;
  onCancel?: VoidFunction;
}

export const SubscriptionCronCreationDialogContent = memo(
  ({
    subscriptionId,
    onCancel,
  }: SubscriptionCronCreationDialogContentProps) => {
    const navigate = useNavigate();

    const handleCreationComplete = useCallback(
      (payload: SubscriptionCronCreationViewCompletePayload) => {
        navigate({
          to: '/tasks/cron/detail/$id',
          params: {
            id: `${payload.id}`,
          },
        });
      },
      [navigate]
    );

    return (
      <DialogContent
        onAbort={onCancel}
        className="flex max-h-[80vh] flex-col overflow-y-auto xl:max-w-2xl"
      >
        <DialogHeader className="sticky">
          <DialogTitle>Create Cron</DialogTitle>
          <DialogDescription>
            Create a cron to execute the subscription.
          </DialogDescription>
        </DialogHeader>
        <div className="min-h-0 flex-1 overflow-y-auto">
          <SubscriptionCronCreationView
            subscriptionId={subscriptionId}
            onComplete={handleCreationComplete}
          />
        </div>
      </DialogContent>
    );
  }
);
