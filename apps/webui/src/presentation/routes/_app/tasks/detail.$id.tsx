import { DetailCardSkeleton } from '@/components/detail-card-skeleton';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { DetailEmptyView } from '@/components/ui/detail-empty-view';
import { Label } from '@/components/ui/label';
import { QueryErrorView } from '@/components/ui/query-error-view';
import { Separator } from '@/components/ui/separator';
import { GET_TASKS, RETRY_TASKS } from '@/domains/recorder/schema/tasks';
import { getApolloQueryError } from '@/infra/errors/apollo';
import { apolloErrorToMessage } from '@/infra/errors/apollo';
import {
  type GetTasksQuery,
  type GetTasksQueryVariables,
  type RetryTasksMutation,
  type RetryTasksMutationVariables,
  SubscriberTaskStatusEnum,
} from '@/infra/graphql/gql/graphql';
import type { RouteStateDataOption } from '@/infra/routes/traits';
import { useMutation, useQuery } from '@apollo/client';
import {
  createFileRoute,
  useCanGoBack,
  useNavigate,
  useRouter,
} from '@tanstack/react-router';
import { format } from 'date-fns';
import { ArrowLeft, RefreshCw } from 'lucide-react';
import { toast } from 'sonner';
import { prettyTaskType } from './-pretty-task-type';
import { getStatusBadge } from './-status-badge';

export const Route = createFileRoute('/_app/tasks/detail/$id')({
  component: TaskDetailRouteComponent,
  staticData: {
    breadcrumb: { label: 'Detail' },
  } satisfies RouteStateDataOption,
});

function TaskDetailRouteComponent() {
  const { id } = Route.useParams();
  const navigate = useNavigate();
  const router = useRouter();
  const canGoBack = useCanGoBack();

  const handleBack = () => {
    if (canGoBack) {
      router.history.back();
    } else {
      navigate({
        to: '/tasks/manage',
      });
    }
  };

  const { data, loading, error, refetch } = useQuery<
    GetTasksQuery,
    GetTasksQueryVariables
  >(GET_TASKS, {
    variables: {
      filters: {
        id: {
          eq: id,
        },
      },
      pagination: {
        page: {
          page: 0,
          limit: 1,
        },
      },
      orderBy: {},
    },
    pollInterval: 5000, // Auto-refresh every 5 seconds for running tasks
  });

  const task = data?.subscriberTasks?.nodes?.[0];

  const [retryTasks] = useMutation<
    RetryTasksMutation,
    RetryTasksMutationVariables
  >(RETRY_TASKS, {
    onCompleted: async () => {
      const refetchResult = await refetch();
      const error = getApolloQueryError(refetchResult);
      if (error) {
        toast.error('Failed to retry task', {
          description: apolloErrorToMessage(error),
        });
        return;
      }
      toast.success('Task retried successfully');
    },
    onError: (error) => {
      toast.error('Failed to retry task', {
        description: apolloErrorToMessage(error),
      });
    },
  });

  if (loading) {
    return <DetailCardSkeleton />;
  }

  if (error) {
    return <QueryErrorView message={error.message} onRetry={refetch} />;
  }

  if (!task) {
    return <DetailEmptyView message="Task not found" />;
  }

  return (
    <div className="container mx-auto max-w-4xl py-6">
      <div className="mb-6 flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Button
            variant="ghost"
            size="sm"
            onClick={handleBack}
            className="h-8 w-8 p-0"
          >
            <ArrowLeft className="h-4 w-4" />
          </Button>
          <div>
            <h1 className="font-bold text-2xl">Task Detail</h1>
            <p className="mt-1 text-muted-foreground">View task #{task.id}</p>
          </div>
        </div>

        <Button variant="outline" size="sm" onClick={() => refetch()}>
          <RefreshCw className="h-4 w-4" />
          Refresh
        </Button>
      </div>

      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Task Information</CardTitle>
              <CardDescription className="mt-2">
                View task execution details
              </CardDescription>
            </div>
            <div className="flex items-center gap-2">
              {getStatusBadge(task.status)}
              {task.status ===
                (SubscriberTaskStatusEnum.Killed ||
                  SubscriberTaskStatusEnum.Failed) && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() =>
                    retryTasks({
                      variables: { filters: { id: { eq: task.id } } },
                    })
                  }
                >
                  Retry
                </Button>
              )}
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="space-y-6">
            {/* Basic Information */}
            <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
              <div className="space-y-2">
                <Label className="font-medium text-sm">Task ID</Label>
                <div className="rounded-md bg-muted p-3">
                  <code className="text-sm">{task.id}</code>
                </div>
              </div>

              <div className="space-y-2">
                <Label className="font-medium text-sm">Task Type</Label>
                <div className="rounded-md bg-muted p-3">
                  <Badge variant="secondary" className="capitalize">
                    {prettyTaskType(task.taskType)}
                  </Badge>
                </div>
              </div>

              <div className="space-y-2">
                <Label className="font-medium text-sm">Priority</Label>
                <div className="rounded-md bg-muted p-3">
                  <span className="text-sm">{task.priority}</span>
                </div>
              </div>

              <div className="space-y-2">
                <Label className="font-medium text-sm">Attempts</Label>
                <div className="rounded-md bg-muted p-3">
                  <span className="text-sm">
                    {task.attempts} / {task.maxAttempts}
                  </span>
                </div>
              </div>

              <div className="space-y-2">
                <Label className="font-medium text-sm">
                  Scheduled Run Time
                </Label>
                <div className="rounded-md bg-muted p-3">
                  <span className="text-sm">
                    {format(new Date(task.runAt), 'yyyy-MM-dd HH:mm:ss')}
                  </span>
                </div>
              </div>

              <div className="space-y-2">
                <Label className="font-medium text-sm">Done Time</Label>
                <div className="rounded-md bg-muted p-3">
                  <span className="text-sm">
                    {task.doneAt
                      ? format(new Date(task.doneAt), 'yyyy-MM-dd HH:mm:ss')
                      : '-'}
                  </span>
                </div>
              </div>

              <div className="space-y-2">
                <Label className="font-medium text-sm">Lock Time</Label>
                <div className="rounded-md bg-muted p-3">
                  <span className="text-sm">
                    {task.lockAt
                      ? format(new Date(task.lockAt), 'yyyy-MM-dd HH:mm:ss')
                      : '-'}
                  </span>
                </div>
              </div>

              <div className="space-y-2">
                <Label className="font-medium text-sm">Lock By</Label>
                <div className="rounded-md bg-muted p-3">
                  <code className="text-sm">{task.lockBy || '-'}</code>
                </div>
              </div>
            </div>

            {/* Job Details */}
            {task.job && (
              <>
                <Separator />
                <div className="space-y-2">
                  <Label className="font-medium text-sm">Job Details</Label>
                  <div className="rounded-md bg-muted p-3">
                    <pre className="overflow-x-auto whitespace-pre-wrap text-sm">
                      <code>{JSON.stringify(task.job, null, 2)}</code>
                    </pre>
                  </div>
                </div>
              </>
            )}

            {/* Error Information */}
            {(task.status === SubscriberTaskStatusEnum.Failed ||
              task.status === SubscriberTaskStatusEnum.Killed) &&
              task.lastError && (
                <>
                  <Separator />
                  <div className="space-y-2">
                    <Label className="font-medium text-sm">Last Error</Label>
                    <div className="rounded-md bg-destructive/10 p-3">
                      <p className="text-destructive text-sm">
                        {task.lastError}
                      </p>
                    </div>
                  </div>
                </>
              )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
