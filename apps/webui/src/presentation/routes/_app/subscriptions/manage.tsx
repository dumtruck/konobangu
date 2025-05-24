import { DataTablePagination } from '@/components/ui/data-table-pagination';
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
} from '@/domains/recorder/graphql/subscriptions';
import type {
  GetSubscriptionsQuery,
  SubscriptionsUpdateInput,
} from '@/infra/graphql/gql/graphql';
import type { RouteStateDataOption } from '@/infra/routes/traits';
import { useDebouncedSkeleton } from '@/presentation/hooks/use-debounded-skeleton';
import { useEvent } from '@/presentation/hooks/use-event';
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
import { useMemo, useState } from 'react';
import { toast } from 'sonner';
import { DataTableRowActions } from '../../../../components/ui/data-table-row-actions';

export const Route = createFileRoute('/_app/subscriptions/manage')({
  component: SubscriptionManageRouteComponent,
  staticData: {
    breadcrumb: { label: 'Manage' },
  } satisfies RouteStateDataOption,
});

function SubscriptionManageRouteComponent() {
  const navigate = useNavigate();

  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({});
  const [sorting, setSorting] = useState<SortingState>([]);
  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: 0,
    pageSize: 10,
  });

  const { loading, error, data, refetch } = useQuery<GetSubscriptionsQuery>(
    GET_SUBSCRIPTIONS,
    {
      variables: {
        page: {
          page: pagination.pageIndex,
          limit: pagination.pageSize,
        },
        filters: {},
        orderBy: {},
      },
      refetchWritePolicy: 'overwrite',
      nextFetchPolicy: 'network-only',
    }
  );
  const [updateSubscription] = useMutation(UPDATE_SUBSCRIPTIONS);
  const [deleteSubscription] = useMutation(DELETE_SUBSCRIPTIONS);
  const { showSkeleton } = useDebouncedSkeleton({ loading });

  const subscriptions = data?.subscriptions;

  const handleUpdateRecord = useEvent(
    (row: Row<SubscriptionDto>) => async (data: SubscriptionsUpdateInput) => {
      const result = await updateSubscription({
        variables: {
          data,
          filters: {
            id: {
              eq: row.original.id,
            },
          },
        },
      });
      if (result.errors) {
        toast.error(result.errors[0].message);
        return;
      }
      const refetchResult = await refetch();
      if (refetchResult.errors) {
        toast.error(refetchResult.errors[0].message);
        return;
      }
      toast.success('Subscription updated');
    }
  );

  const handleDeleteRecord = useEvent(
    (row: Row<SubscriptionDto>) => async () => {
      const result = await deleteSubscription({
        variables: { filters: { id: { eq: row.original.id } } },
      });
      if (result.errors) {
        toast.error(result.errors[0].message);
        return;
      }
      const refetchResult = await refetch();
      if (refetchResult.errors) {
        toast.error(refetchResult.errors[0].message);
        return;
      }
      toast.success('Subscription deleted');
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
                to: '/subscriptions/detail/$subscriptionId',
                params: { subscriptionId: `${row.original.id}` },
              });
            }}
            onEdit={() => {
              navigate({
                to: '/subscriptions/edit/$subscriptionId',
                params: { subscriptionId: `${row.original.id}` },
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
    state: {
      pagination,
      sorting,
      columnVisibility,
    },
  });

  if (error) {
    return <QueryErrorView message={error.message} onRetry={refetch} />;
  }

  return (
    <div className="container mx-auto space-y-4 rounded-md">
      <div className="flex items-center py-4">
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
                    {row.getVisibleCells().map((cell) => (
                      <TableCell key={cell.id}>
                        {flexRender(
                          cell.column.columnDef.cell,
                          cell.getContext()
                        )}
                      </TableCell>
                    ))}
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
