import { AppNotFoundComponent } from '@/views/components/layout/app-not-found';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/404')({
  component: AppNotFoundComponent,
});
