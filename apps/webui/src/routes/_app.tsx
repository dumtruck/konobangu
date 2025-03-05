import { createFileRoute } from '@tanstack/solid-router';
import { beforeLoadGuard } from '~/auth/guard';
import { AppLayout } from '~/components/layout/app-layout';

export const Route = createFileRoute('/_app')({
  component: AppLayout,
  beforeLoad: beforeLoadGuard,
});
