import { createFileRoute } from '@tanstack/solid-router';
import { buildLeafRouteStaticData } from '~/utils/route';

export const Route = createFileRoute('/_app/settings/downloader')({
  component: SettingsDownloaderRouteComponent,
  staticData: buildLeafRouteStaticData({
    title: 'Downloader',
  }),
});

function SettingsDownloaderRouteComponent() {
  return <div>Hello "/_app/settings/downloader"!</div>;
}
