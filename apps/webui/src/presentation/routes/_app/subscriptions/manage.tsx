import { Button } from '@/components/ui/button';
import { DataTablePagination } from '@/components/ui/data-table-pagination';
import { DataTableRowActions } from '@/components/ui/data-table-row-actions';
import { DataTableViewOptions } from '@/components/ui/data-table-view-options';
import { QueryErrorView } from '@/components/ui/query-error-view';
import { Skeleton } from '@/components/ui/skeleton';
import { Switch } from '@/components/ui/switch';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import {
  DELETE_SUBSCRIPTIONS,
  GET_SUBSCRIPTIONS,
  type SubscriptionDto,
  UPDATE_SUBSCRIPTIONS,
} from '@/domains/recorder/schema/subscriptions';
import type {
  GetSubscriptionsQuery,
  SubscriptionsUpdateInput,
} from '@/infra/graphql/gql/graphql';
import type { RouteStateDataOption } from '@/infra/routes/traits';
import { useDebouncedSkeleton } from '@/presentation/hooks/use-debounded-skeleton';
import { useEvent } from '@/presentation/hooks/use-event';
import { cn } from '@/presentation/utils';
import { useMutation, useQuery } from '@apollo/client';
import { createFileRoute } from '@tanstack/react-router';
import { useNavigate } from '@tanstack/react-router';
import {
  type ColumnDef,
  type PaginationState,
  type Row,
  type SortingState,
  type VisibilityState,
  flexRender,
  getCoreRowModel,
  getPaginationRowModel,
  useReactTable,
} from '@tanstack/react-table';
import { format } from 'date-fns';
import { Plus } from 'lucide-react';
import { useMemo, useState } from 'react';
import { toast } from 'sonner';

export const Route = createFileRoute('/_app/subscriptions/manage')({
  component: SubscriptionManageRouteComponent,
  staticData: {
    breadcrumb: { label: 'Manage' },
  } satisfies RouteStateDataOption,
});

function SubscriptionManageRouteComponent() {
  const navigate = useNavigate();

  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({
    createdAt: false,
    updatedAt: false,
  });
  const [sorting, setSorting] = useState<SortingState>([]);
  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: 0,
    pageSize: 10,
  });

  const { loading, error, data, refetch } = useQuery<GetSubscriptionsQuery>(
    GET_SUBSCRIPTIONS,
    {
      variables: {
        pagination: {
          page: {
            page: pagination.pageIndex + 1,
            limit: pagination.pageSize,
          },
        },
        filters: {},
        orderBy: {
          updatedAt: 'DESC',
        },
      },
      refetchWritePolicy: 'overwrite',
      nextFetchPolicy: 'network-only',
    }
  );
  const [updateSubscription] = useMutation(UPDATE_SUBSCRIPTIONS, {
    onCompleted: async () => {
      const refetchResult = await refetch();
      if (refetchResult.errors) {
        toast.error(refetchResult.errors[0].message);
        return;
      }
      toast.success('Subscription updated');
    },
    onError: (error) => {
      toast.error('Failed to update subscription', {
        description: error.message,
      });
    },
  });
  const [deleteSubscription] = useMutation(DELETE_SUBSCRIPTIONS, {
    onCompleted: async () => {
      const refetchResult = await refetch();
      if (refetchResult.errors) {
        toast.error(refetchResult.errors[0].message);
        return;
      }
      toast.success('Subscription deleted');
    },
    onError: (error) => {
      toast.error('Failed to delete subscription', {
        description: error.message,
      });
    },
  });
  const { showSkeleton } = useDebouncedSkeleton({ loading });

  const subscriptions = data?.subscriptions;

  const handleUpdateRecord = useEvent(
    (row: Row<SubscriptionDto>) => async (data: SubscriptionsUpdateInput) => {
      await updateSubscription({
        variables: {
          data,
          filters: {
            id: {
              eq: row.original.id,
            },
          },
        },
      });
    }
  );

  const handleDeleteRecord = useEvent(
    (row: Row<SubscriptionDto>) => async () => {
      await deleteSubscription({
        variables: { filters: { id: { eq: row.original.id } } },
      });
    }
  );

  const columns = useMemo(() => {
    const cs: ColumnDef<SubscriptionDto>[] = [
      {
        header: 'Enabled',
        accessorKey: 'enabled',
        cell: ({ row }) => {
          const enabled = row.original.enabled;
          return (
            <div className="px-1">
              <Switch
                checked={enabled}
                onCheckedChange={(enabled) =>
                  handleUpdateRecord(row)({ enabled: enabled })
                }
              />
            </div>
          );
        },
        enableResizing: true,
      },
      {
        header: 'Name',
        accessorKey: 'displayName',
        cell: ({ row }) => {
          const displayName = row.original.displayName;
          return (
            <div className="whitespace-normal break-words">{displayName}</div>
          );
        },
      },
      {
        header: 'Category',
        accessorKey: 'category',
      },
      {
        header: 'Source URL',
        accessorKey: 'sourceUrl',
        cell: ({ row }) => {
          const sourceUrl = row.original.sourceUrl;
          return (
            <div className="whitespace-normal break-words">{sourceUrl}</div>
          );
        },
      },
      {
        header: 'Created At',
        accessorKey: 'createdAt',
        cell: ({ row }) => {
          const createdAt = row.original.createdAt;
          return (
            <div className="text-sm">
              {format(new Date(createdAt), 'yyyy-MM-dd HH:mm:ss')}
            </div>
          );
        },
      },
      {
        header: 'Updated At',
        accessorKey: 'updatedAt',
        cell: ({ row }) => {
          const updatedAt = row.original.updatedAt;
          return (
            <div className="text-sm">
              {format(new Date(updatedAt), 'yyyy-MM-dd HH:mm:ss')}
            </div>
          );
        },
      },
      {
        id: 'actions',
        cell: ({ row }) => (
          <DataTableRowActions
            row={row}
            getId={(row) => row.original.id}
            showDetail
            showEdit
            showDelete
            onDetail={() => {
              navigate({
                to: '/subscriptions/detail/$id',
                params: { id: `${row.original.id}` },
              });
            }}
            onEdit={() => {
              navigate({
                to: '/subscriptions/edit/$id',
                params: { id: `${row.original.id}` },
              });
            }}
            onDelete={handleDeleteRecord(row)}
          />
        ),
      },
    ];
    return cs;
  }, [handleUpdateRecord, handleDeleteRecord, navigate]);

  const table = useReactTable({
    data: data?.subscriptions?.nodes ?? [],
    columns,
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    onPaginationChange: setPagination,
    onSortingChange: setSorting,
    onColumnVisibilityChange: setColumnVisibility,
    pageCount: subscriptions?.paginationInfo?.pages,
    rowCount: subscriptions?.paginationInfo?.total,
    enableColumnPinning: true,
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
    <div className="container mx-auto space-y-4 rounded-md">
      <div className="flex items-center justify-between pt-4">
        <div>
          <h1 className="font-bold text-2xl">Subscription Management</h1>
          <p className="text-muted-foreground">Manage your subscription</p>
        </div>
        <Button onClick={() => navigate({ to: '/subscriptions/create' })}>
          <Plus className="mr-2 h-4 w-4" />
          Add Subscription
        </Button>
      </div>
      <div className="flex items-center py-2">
        <DataTableViewOptions table={table} />
      </div>
      <div className="rounded-md border">
        <Table>
          <TableHeader>
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
                {headerGroup.headers.map((header) => {
                  return (
                    <TableHead key={header.id}>
                      {header.isPlaceholder
                        ? null
                        : flexRender(
                            header.column.columnDef.header,
                            header.getContext()
                          )}
                    </TableHead>
                  );
                })}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {showSkeleton &&
              Array.from(new Array(pagination.pageSize)).map((_, index) => (
                <TableRow key={index}>
                  {table.getVisibleLeafColumns().map((column) => (
                    <TableCell key={column.id}>
                      <Skeleton className="h-8" />
                    </TableCell>
                  ))}
                </TableRow>
              ))}
            {!showSkeleton &&
              (table.getRowModel().rows?.length ? (
                table.getRowModel().rows.map((row) => (
                  <TableRow
                    key={row.id}
                    data-state={row.getIsSelected() && 'selected'}
                  >
                    {row.getVisibleCells().map((cell) => {
                      const isPinned = cell.column.getIsPinned();
                      return (
                        <TableCell
                          key={cell.id}
                          className={cn({
                            'sticky z-1 bg-background shadow-xs': isPinned,
                            'right-0': isPinned === 'right',
                            'left-0': isPinned === 'left',
                          })}
                        >
                          {flexRender(
                            cell.column.columnDef.cell,
                            cell.getContext()
                          )}
                        </TableCell>
                      );
                    })}
                  </TableRow>
                ))
              ) : (
                <TableRow>
                  <TableCell
                    colSpan={columns.length}
                    className="h-24 text-center"
                  >
                    No results.
                  </TableCell>
                </TableRow>
              ))}
          </TableBody>
        </Table>
      </div>
      <DataTablePagination table={table} showSelectedRowCount={false} />
    </div>
  );
}
