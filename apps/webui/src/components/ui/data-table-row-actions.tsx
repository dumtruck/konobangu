"use client";

import type { Row } from "@tanstack/react-table";
import { MoreHorizontal } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

import { PropsWithChildren, useMemo } from "react";

interface DataTableRowActionsProps<DataView, Id> {
  row: Row<DataView>;
  getId: (row: Row<DataView>) => Id;
  showDetail?: boolean;
  showEdit?: boolean;
  showDelete?: boolean;
  onDetail?: (id: Id) => void;
  onDelete?: (id: Id) => void;
  onEdit?: (id: Id) => void;
  modal?: boolean;
}

export function DataTableRowActions<DataView, Id>({
  row,
  getId,
  showDetail,
  showDelete,
  showEdit,
  onDetail,
  onDelete,
  onEdit,
  children,
  modal,
}: PropsWithChildren<DataTableRowActionsProps<DataView, Id>>) {
  const id = useMemo(() => getId(row), [getId, row]);
  return (
    <DropdownMenu modal={modal}>
      <DropdownMenuTrigger asChild>
        <Button
          variant="ghost"
          className="flex h-8 w-8 p-0 data-[state=open]:bg-muted"
        >
          <MoreHorizontal />
          <span className="sr-only">Open menu</span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-[160px]">
        {children}
        {showDetail && (
          <DropdownMenuItem onClick={() => onDetail?.(id)}>
            Detail
          </DropdownMenuItem>
        )}
        {showEdit && (
          <DropdownMenuItem onClick={() => onEdit?.(id)}>Edit</DropdownMenuItem>
        )}
        {(showDetail || showEdit) && showDelete && <DropdownMenuSeparator />}
        {showDelete && (
          <DropdownMenuItem onClick={() => onDelete?.(id)}>
            Delete
            <DropdownMenuShortcut>⌘⌫</DropdownMenuShortcut>
          </DropdownMenuItem>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
