import { buildLeafRouteStaticData } from '@/utils/route';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_app/settings/downloader')({
  component: SettingsDownloaderRouteComponent,
  staticData: buildLeafRouteStaticData({
    title: 'Downloader',
  }),
});

function SettingsDownloaderRouteComponent() {
  return <div>Hello "/_app/settings/downloader"!</div>;
}
