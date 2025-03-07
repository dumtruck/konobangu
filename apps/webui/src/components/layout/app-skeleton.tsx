export function AppSkeleton() {
  return (
    <div class="flex min-h-svh flex-1 flex-col gap-4 md:min-h-0">
      <div class="grid auto-rows-min gap-4 md:grid-cols-3">
        <div class="aspect-video rounded-xl bg-muted/50" />
        <div class="aspect-video rounded-xl bg-muted/50" />
        <div class="aspect-video rounded-xl bg-muted/50" />
      </div>
      <div class="min-h-svh flex-1 rounded-xl bg-muted/50 md:min-h-0" />
    </div>
  );
}
