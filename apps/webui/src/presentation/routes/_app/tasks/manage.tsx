import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { DataTablePagination } from '@/components/ui/data-table-pagination';
import { DataTableRowActions } from '@/components/ui/data-table-row-actions';
import { DetailEmptyView } from '@/components/ui/detail-empty-view';
import { QueryErrorView } from '@/components/ui/query-error-view';
import { Skeleton } from '@/components/ui/skeleton';
import { GET_TASKS, type TaskDto } from '@/domains/recorder/schema/tasks';
import type { GetTasksQuery } from '@/infra/graphql/gql/graphql';
import type { RouteStateDataOption } from '@/infra/routes/traits';
import { useDebouncedSkeleton } from '@/presentation/hooks/use-debounded-skeleton';
import { useQuery } from '@apollo/client';
import { createFileRoute, useNavigate } from '@tanstack/react-router';
import {
  type ColumnDef,
  type PaginationState,
  type SortingState,
  type VisibilityState,
  getCoreRowModel,
  getPaginationRowModel,
  useReactTable,
} from '@tanstack/react-table';
import { format } from 'date-fns';
import {
  AlertCircle,
  CheckCircle,
  Clock,
  Loader2,
  RefreshCw,
} from 'lucide-react';
import { useMemo, useState } from 'react';

export const Route = createFileRoute('/_app/tasks/manage')({
  component: TaskManageRouteComponent,
  staticData: {
    breadcrumb: { label: 'Manage' },
  } satisfies RouteStateDataOption,
});

function TaskManageRouteComponent() {
  const navigate = useNavigate();

  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({
    lockAt: false,
    lockBy: false,
    attempts: false,
  });
  const [sorting, setSorting] = useState<SortingState>([]);
  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: 0,
    pageSize: 10,
  });

  const { loading, error, data, refetch } = useQuery<GetTasksQuery>(GET_TASKS, {
    variables: {
      pagination: {
        page: {
          page: pagination.pageIndex,
          limit: pagination.pageSize,
        },
      },
      filters: {},
      orderBy: {
        runAt: 'DESC',
      },
    },
    pollInterval: 5000, // Auto-refresh every 5 seconds
  });

  const { showSkeleton } = useDebouncedSkeleton({ loading });

  const tasks = data?.subscriberTasks;

  const columns = useMemo(() => {
    const cs: ColumnDef<TaskDto>[] = [
      {
        header: 'ID',
        accessorKey: 'id',
        cell: ({ row }) => {
          return (
            <div
              className="max-w-[200px] truncate font-mono text-sm"
              title={row.original.id}
            >
              {row.original.id}
            </div>
          );
        },
      },
      {
        header: 'Status',
        accessorKey: 'status',
        cell: ({ row }) => {
          return getStatusBadge(row.original.status);
        },
      },
      {
        header: 'Priority',
        accessorKey: 'priority',
        cell: ({ row }) => {
          return getPriorityBadge(row.original.priority);
        },
      },
      {
        header: 'Attempts',
        accessorKey: 'attempts',
        cell: ({ row }) => {
          const attempts = row.original.attempts;
          const maxAttempts = row.original.maxAttempts;
          return (
            <div className="text-sm">
              {attempts} / {maxAttempts}
            </div>
          );
        },
      },
      {
        header: 'Run At',
        accessorKey: 'runAt',
        cell: ({ row }) => {
          const runAt = row.original.runAt;
          return (
            <div className="text-sm">
              {format(new Date(runAt), 'yyyy-MM-dd HH:mm:ss')}
            </div>
          );
        },
      },
      {
        header: 'Done At',
        accessorKey: 'doneAt',
        cell: ({ row }) => {
          const doneAt = row.original.doneAt;
          return (
            <div className="text-sm">
              {doneAt ? format(new Date(doneAt), 'yyyy-MM-dd HH:mm:ss') : '-'}
            </div>
          );
        },
      },
      {
        header: 'Last Error',
        accessorKey: 'lastError',
        cell: ({ row }) => {
          const lastError = row.original.lastError;
          return (
            <div
              className="max-w-xs truncate text-sm"
              title={lastError || undefined}
            >
              {lastError || '-'}
            </div>
          );
        },
      },
      {
        header: 'Lock At',
        accessorKey: 'lockAt',
        cell: ({ row }) => {
          const lockAt = row.original.lockAt;
          return (
            <div className="text-sm">
              {lockAt ? format(new Date(lockAt), 'yyyy-MM-dd HH:mm:ss') : '-'}
            </div>
          );
        },
      },
      {
        header: 'Lock By',
        accessorKey: 'lockBy',
        cell: ({ row }) => {
          const lockBy = row.original.lockBy;
          return <div className="font-mono text-sm">{lockBy || '-'}</div>;
        },
      },
      {
        id: 'actions',
        cell: ({ row }) => (
          <DataTableRowActions
            row={row}
            getId={(row) => row.original.id}
            showDetail
            onDetail={() => {
              navigate({
                to: '/tasks/detail/$id',
                params: { id: row.original.id },
              });
            }}
          />
        ),
      },
    ];
    return cs;
  }, [navigate]);

  const table = useReactTable({
    data: useMemo(() => (tasks?.nodes ?? []) as TaskDto[], [tasks]),
    columns,
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    onPaginationChange: setPagination,
    onSortingChange: setSorting,
    onColumnVisibilityChange: setColumnVisibility,
    pageCount: tasks?.paginationInfo?.pages,
    rowCount: tasks?.paginationInfo?.total,
    enableColumnPinning: true,
    autoResetPageIndex: true,
    manualPagination: true,
    state: {
      pagination,
      sorting,
      columnVisibility,
    },
    initialState: {
      columnPinning: {
        right: ['actions'],
      },
    },
  });

  if (error) {
    return <QueryErrorView message={error.message} onRetry={refetch} />;
  }

  return (
    <div className="container mx-auto space-y-4 px-4">
      <div className="flex items-center justify-between pt-4">
        <div>
          <h1 className="font-bold text-2xl">Subscription Management</h1>
          <p className="text-muted-foreground">Manage your subscription</p>
        </div>
        <Button onClick={() => refetch()} variant="outline" size="sm">
          <RefreshCw className="h-4 w-4" />
        </Button>
      </div>

      <div className="space-y-3">
        {showSkeleton &&
          Array.from(new Array(10)).map((_, index) => (
            <Skeleton key={index} className="h-32 w-full" />
          ))}

        {!showSkeleton && table.getRowModel().rows?.length > 0 ? (
          table.getRowModel().rows.map((row) => {
            const task = row.original;
            return (
              <div
                key={task.id}
                className="space-y-3 rounded-lg border bg-card p-4"
              >
                {/* Header with status and priority */}
                <div className="flex items-center justify-between gap-2">
                  <div className="font-mono text-muted-foreground text-xs">
                    # {task.id}
                  </div>
                  <div className="flex gap-2">
                    <Badge variant="outline">{task.taskType}</Badge>
                  </div>
                </div>
                <div className="mt-1 flex items-center gap-2">
                  {getStatusBadge(task.status)}
                  <div className="mr-0 ml-auto">
                    <DataTableRowActions
                      row={row}
                      getId={(r) => r.original.id}
                      showDetail
                      onDetail={() => {
                        navigate({
                          to: '/tasks/detail/$id',
                          params: { id: task.id },
                        });
                      }}
                    />
                  </div>
                </div>

                {task.job && (
                  <div className="text-sm">
                    <span className="text-muted-foreground">Job: </span>
                    <br />
                    <span
                      className="whitespace-pre-wrap"
                      dangerouslySetInnerHTML={{
                        __html: JSON.stringify(task.job, null, 2),
                      }}
                    />
                  </div>
                )}

                {/* Time info */}
                <div className="grid grid-cols-2 gap-2 text-sm">
                  <div>
                    <span className="text-muted-foreground">Run at: </span>
                    <span>{format(new Date(task.runAt), 'MM/dd HH:mm')}</span>
                  </div>

                  <div>
                    <span className="text-muted-foreground">Done: </span>
                    <span>
                      {task.doneAt
                        ? format(new Date(task.doneAt), 'MM/dd HH:mm')
                        : '-'}
                    </span>
                  </div>

                  {/* Attempts */}
                  <div className="text-sm">
                    <span className="text-muted-foreground">Attempts: </span>
                    <span>
                      {task.attempts} / {task.maxAttempts}
                    </span>
                  </div>

                  {/* Priority */}
                  <div className="text-sm">
                    <span className="text-muted-foreground">Priority: </span>
                    <span>{task.priority}</span>
                  </div>
                </div>

                {/* Error if exists */}
                {task.status === 'error' && task.lastError && (
                  <div className="rounded bg-destructive/10 p-2 text-destructive text-sm">
                    {task.lastError}
                  </div>
                )}
              </div>
            );
          })
        ) : (
          <DetailEmptyView message="No tasks found" />
        )}
      </div>

      <DataTablePagination table={table} showSelectedRowCount={false} />
    </div>
  );
}

function getStatusBadge(status: string) {
  switch (status.toLowerCase()) {
    case 'completed':
    case 'done':
      return (
        <Badge variant="secondary" className="bg-green-100 text-green-800">
          <CheckCircle className="mr-1 h-3 w-3" />
          Completed
        </Badge>
      );
    case 'running':
    case 'active':
      return (
        <Badge variant="secondary" className="bg-blue-100 text-blue-800">
          <Loader2 className="mr-1 h-3 w-3 animate-spin" />
          Running
        </Badge>
      );
    case 'failed':
    case 'error':
      return (
        <Badge variant="destructive">
          <AlertCircle className="mr-1 h-3 w-3" />
          Failed
        </Badge>
      );
    case 'pending':
    case 'waiting':
      return (
        <Badge variant="secondary" className="bg-yellow-100 text-yellow-800">
          <Clock className="mr-1 h-3 w-3" />
          Pending
        </Badge>
      );
    default:
      return <Badge variant="outline">{status}</Badge>;
  }
}
