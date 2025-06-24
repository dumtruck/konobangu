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
import { Dialog, DialogTrigger } from '@/components/ui/dialog';
import { Img } from '@/components/ui/img';
import { Label } from '@/components/ui/label';
import { QueryErrorView } from '@/components/ui/query-error-view';
import { Separator } from '@/components/ui/separator';
import { DELETE_FEED, INSERT_FEED } from '@/domains/recorder/schema/feeds';
import { GET_SUBSCRIPTION_DETAIL } from '@/domains/recorder/schema/subscriptions';
import { SubscriptionService } from '@/domains/recorder/services/subscription.service';
import { useInject } from '@/infra/di/inject';
import {
  apolloErrorToMessage,
  getApolloQueryError,
} from '@/infra/errors/apollo';
import {
  type DeleteFeedMutation,
  type DeleteFeedMutationVariables,
  FeedSourceEnum,
  FeedTypeEnum,
  type GetSubscriptionDetailQuery,
  type InsertFeedMutation,
  type InsertFeedMutationVariables,
  SubscriptionCategoryEnum,
} from '@/infra/graphql/gql/graphql';
import { useMutation, useQuery } from '@apollo/client';
import {
  createFileRoute,
  useCanGoBack,
  useNavigate,
  useRouter,
} from '@tanstack/react-router';
import { format } from 'date-fns';
import {
  ArrowLeft,
  Edit,
  ExternalLink,
  ListIcon,
  PlusIcon,
  RefreshCcwIcon,
  Trash2,
} from 'lucide-react';
import { useMemo } from 'react';
import { toast } from 'sonner';
import { prettyTaskType } from '../tasks/-pretty-task-type';
import { SubscriptionSyncDialogContent } from './-sync';

export const Route = createFileRoute('/_app/subscriptions/detail/$id')({
  component: SubscriptionDetailRouteComponent,
});

function SubscriptionDetailRouteComponent() {
  const { id } = Route.useParams();
  const navigate = useNavigate();
  const router = useRouter();
  const canGoBack = useCanGoBack();
  const subscriptionService = useInject(SubscriptionService);

  const handleBack = () => {
    if (canGoBack) {
      router.history.back();
    } else {
      navigate({
        to: '/subscriptions/manage',
      });
    }
  };

  const handleReload = async () => {
    const result = await refetch();
    const error = getApolloQueryError(result);
    if (error) {
      toast.error('Failed to reload subscription', {
        description: apolloErrorToMessage(error),
      });
    }
  };

  const { data, loading, error, refetch } =
    useQuery<GetSubscriptionDetailQuery>(GET_SUBSCRIPTION_DETAIL, {
      variables: {
        id: Number.parseInt(id),
      },
    });

  const handleEnterEditMode = () => {
    navigate({
      to: '/subscriptions/edit/$id',
      params: {
        id,
      },
    });
  };

  const [insertFeed] = useMutation<
    InsertFeedMutation,
    InsertFeedMutationVariables
  >(INSERT_FEED, {
    onCompleted: async () => {
      const result = await refetch();
      const error = getApolloQueryError(result);
      if (error) {
        toast.error('Failed to add feed', {
          description: apolloErrorToMessage(error),
        });
        return;
      }
      toast.success('Feed added');
    },
    onError: (error) => {
      toast.error('Failed to add feed', {
        description: apolloErrorToMessage(error),
      });
    },
  });

  const [deleteFeed] = useMutation<
    DeleteFeedMutation,
    DeleteFeedMutationVariables
  >(DELETE_FEED, {
    onCompleted: async () => {
      const result = await refetch();
      const error = getApolloQueryError(result);
      if (error) {
        toast.error('Failed to delete feed', {
          description: apolloErrorToMessage(error),
        });
        return;
      }
      toast.success('Feed deleted');
    },
    onError: (error) => {
      toast.error('Failed to delete feed', {
        description: apolloErrorToMessage(error),
      });
    },
  });

  const subscription = data?.subscriptions?.nodes?.[0];

  const sourceUrlMeta = useMemo(
    () =>
      subscription
        ? subscriptionService.extractSourceUrlMeta(
            subscription?.category,
            subscription?.sourceUrl
          )
        : null,
    [
      subscription,
      subscription?.category,
      subscription?.sourceUrl,
      subscriptionService.extractSourceUrlMeta,
    ]
  );

  if (loading) {
    return <DetailCardSkeleton />;
  }

  if (error) {
    return <QueryErrorView message={error.message} />;
  }

  if (!subscription) {
    return <DetailEmptyView message="Not found certain subscription" />;
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
            <h1 className="font-bold text-2xl">Subscription detail</h1>
            <p className="mt-1 text-muted-foreground">
              View subscription #{subscription.id}
            </p>
          </div>
        </div>

        <div className="flex gap-2">
          <Button onClick={handleEnterEditMode}>
            <Edit className="mr-2 h-4 w-4" />
            Edit
          </Button>{' '}
        </div>
      </div>

      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Subscription information</CardTitle>
              <CardDescription className="mt-2">
                View subscription detail
              </CardDescription>
            </div>
            <div className="flex gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() =>
                  navigate({
                    to: '/tasks/manage',
                  })
                }
              >
                <ListIcon className="h-4 w-4" />
                Tasks
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="space-y-6">
            <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
              <div className="space-y-2">
                <Label className="font-medium text-sm">ID</Label>
                <div className="rounded-md bg-muted p-3">
                  <code className="text-sm">{subscription.id}</code>
                </div>
              </div>

              <div className="space-y-2">
                <Label className="font-medium text-sm">Category</Label>
                <div className="rounded-md bg-muted p-3">
                  <Badge variant="outline">{subscription.category}</Badge>
                </div>
              </div>

              <div className="space-y-2">
                <Label className="font-medium text-sm">Display Name</Label>
                <div className="rounded-md bg-muted p-3">
                  <span className="text-sm">
                    {subscription.displayName || 'Not set'}
                  </span>
                </div>
              </div>

              <div className="space-y-2">
                <Label className="font-medium text-sm">Enabled</Label>
                <div className="rounded-md bg-muted p-3">
                  <span className="text-sm">
                    {subscription.enabled ? 'Enabled' : 'Disabled'}
                  </span>
                </div>
              </div>

              {subscription.category ===
                SubscriptionCategoryEnum.MikanSeason && (
                <>
                  <div className="space-y-2">
                    <Label className="font-medium text-sm">Credential ID</Label>
                    <div className="rounded-md bg-muted p-3">
                      <code className="text-sm">
                        {subscription.credential3rd?.id ?? '-'}
                      </code>
                    </div>
                  </div>

                  <div className="space-y-2">
                    <Label className="font-medium text-sm">
                      Credential Username
                    </Label>
                    <div className="rounded-md bg-muted p-3">
                      <code className="text-sm">
                        {subscription.credential3rd?.username ?? '-'}
                      </code>
                    </div>
                  </div>
                </>
              )}
            </div>
            <Separator />
            <div className="space-y-2">
              <Label className="font-medium text-sm">Source URL</Label>
              <div className="flex items-center justify-between rounded-md bg-muted p-3">
                <span className="break-all text-sm">
                  {subscription.sourceUrl || '-'}
                </span>
                {subscription.sourceUrl && (
                  <Button
                    variant="ghost"
                    size="sm"
                    className="ml-2 h-6 w-6 p-0"
                    onClick={() =>
                      window.open(subscription.sourceUrl, '_blank')
                    }
                  >
                    <ExternalLink className="h-3 w-3" />
                  </Button>
                )}
              </div>
            </div>
            <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
              {sourceUrlMeta?.category ===
                SubscriptionCategoryEnum.MikanSeason && (
                <>
                  <div className="space-y-2">
                    <Label className="font-medium text-sm">Year</Label>
                    <div className="rounded-md bg-muted p-3">
                      <code className="text-sm">{sourceUrlMeta.year}</code>
                    </div>
                  </div>
                  <div className="space-y-2">
                    <Label className="font-medium text-sm">Season</Label>
                    <div className="rounded-md bg-muted p-3">
                      <code className="text-sm">{sourceUrlMeta.seasonStr}</code>
                    </div>
                  </div>
                </>
              )}
            </div>
            <Separator />

            <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
              <div className="space-y-2">
                <Label className="font-medium text-sm">Created at</Label>
                <div className="rounded-md bg-muted p-3">
                  <span className="text-sm">
                    {format(
                      new Date(subscription.createdAt),
                      'yyyy-MM-dd HH:mm:ss'
                    )}
                  </span>
                </div>
              </div>

              <div className="space-y-2">
                <Label className="font-medium text-sm">Updated at</Label>
                <div className="rounded-md bg-muted p-3">
                  <span className="text-sm">
                    {format(
                      new Date(subscription.updatedAt),
                      'yyyy-MM-dd HH:mm:ss'
                    )}
                  </span>
                </div>
              </div>
            </div>

            <Separator />
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <Label className="font-medium text-sm">Associated Feeds</Label>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() =>
                    insertFeed({
                      variables: {
                        data: {
                          subscriptionId: Number.parseInt(id),
                          feedType: FeedTypeEnum.Rss,
                          feedSource: FeedSourceEnum.SubscriptionEpisode,
                        },
                      },
                    })
                  }
                >
                  <PlusIcon className="h-4 w-4" />
                  Add Feed
                </Button>
              </div>

              <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
                {subscription.feed?.nodes &&
                subscription.feed.nodes.length > 0 ? (
                  subscription.feed.nodes.map((feed) => (
                    <Card
                      key={feed.id}
                      className="group relative cursor-pointer p-4 transition-colors hover:bg-accent/50"
                      onClick={() => {
                        window.open(`/api/feeds/rss/${feed.token}`, '_blank');
                      }}
                    >
                      <div className="flex flex-col space-y-2">
                        <div className="flex items-center justify-between">
                          <Label className="whitespace-nowrap font-medium text-sm capitalize">
                            <span>{feed.feedType} Feed</span>
                          </Label>
                          <Button
                            variant="ghost"
                            size="sm"
                            className="h-6 w-6 p-0 opacity-0 transition-opacity group-hover:opacity-100"
                            onClick={(e) => {
                              e.stopPropagation();
                              deleteFeed({
                                variables: {
                                  filters: {
                                    id: {
                                      eq: feed.id,
                                    },
                                  },
                                },
                              });
                            }}
                          >
                            <Trash2 className="h-3 w-3 text-destructive" />
                          </Button>
                        </div>

                        <code className="break-all rounded bg-muted px-2 py-1 font-mono text-xs">
                          {feed.token}
                        </code>

                        <div className="text-muted-foreground text-xs">
                          {format(new Date(feed.createdAt), 'MM-dd HH:mm')}
                        </div>
                      </div>
                    </Card>
                  ))
                ) : (
                  <div className="col-span-full py-8 text-center text-muted-foreground">
                    No associated feeds now
                  </div>
                )}
              </div>
            </div>

            <Separator />
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <Label className="font-medium text-sm">Associated Tasks</Label>
                <Dialog>
                  <DialogTrigger asChild>
                    <Button variant="outline" size="sm">
                      <RefreshCcwIcon className="h-4 w-4" />
                      Sync
                    </Button>
                  </DialogTrigger>
                  <SubscriptionSyncDialogContent
                    id={subscription.id}
                    onCancel={handleReload}
                  />
                </Dialog>
              </div>
              <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
                {subscription.subscriberTask?.nodes &&
                subscription.subscriberTask.nodes.length > 0 ? (
                  subscription.subscriberTask.nodes.map((task) => (
                    <Card
                      key={task.id}
                      className="group relative cursor-pointer p-4 transition-colors hover:bg-accent/50"
                      onClick={() =>
                        navigate({
                          to: '/tasks/detail/$id',
                          params: {
                            id: task.id,
                          },
                        })
                      }
                    >
                      <div className="flex flex-col space-y-2">
                        <div className="flex items-center justify-between">
                          <Label className="font-medium text-sm capitalize">
                            <span>{prettyTaskType(task.taskType)} Task</span>
                          </Label>
                        </div>

                        <code className="break-all rounded bg-muted px-2 py-1 font-mono text-xs">
                          {task.id}
                        </code>

                        <div className="text-muted-foreground text-xs">
                          {task.status}
                        </div>
                      </div>
                    </Card>
                  ))
                ) : (
                  <div className="col-span-full py-8 text-center text-muted-foreground">
                    No associated tasks now
                  </div>
                )}
              </div>
            </div>

            {subscription.bangumi?.nodes &&
              subscription.bangumi.nodes.length > 0 && (
                <>
                  <Separator />
                  <div className="space-y-4">
                    <Label className="font-medium text-sm">
                      Associated Bangumi
                    </Label>
                    <div className="space-y-3">
                      {subscription.bangumi.nodes.map((bangumi) => (
                        <Card key={bangumi.id} className="p-4">
                          <div className="grid grid-cols-2 gap-4 md:grid-cols-3">
                            <div className="col-span-1 row-span-2 space-y-2">
                              <div className="flex h-full items-center justify-center overflow-hidden rounded-md bg-muted">
                                {bangumi.posterLink && (
                                  <Img
                                    src={`/api/static${bangumi.posterLink}`}
                                    alt="Poster"
                                    className="h-full w-full object-cover"
                                    loading="lazy"
                                  />
                                )}
                              </div>
                            </div>
                            <div className="space-y-2">
                              <Label className="font-medium text-muted-foreground text-xs">
                                Display Name
                              </Label>
                              <div className="text-sm">
                                {bangumi.displayName}
                              </div>
                            </div>
                            <div className="space-y-2">
                              <Label className="font-medium text-muted-foreground text-xs">
                                Fansub
                              </Label>
                              <div className="text-sm">
                                {bangumi.fansub || '-'}
                              </div>
                            </div>
                            <div className="space-y-2">
                              <Label className="font-medium text-muted-foreground text-xs">
                                Season
                              </Label>
                              <div className="text-sm">
                                {bangumi.season || '-'}
                              </div>
                            </div>
                            <div className="space-y-2">
                              <Label className="font-medium text-muted-foreground text-xs">
                                Updated At
                              </Label>
                              <div className="font-mono text-sm">
                                {format(
                                  new Date(bangumi.updatedAt),
                                  'yyyy-MM-dd'
                                )}
                              </div>
                            </div>
                          </div>
                          {bangumi.homepage && (
                            <div className="mt-3 border-t pt-3">
                              <Button
                                variant="outline"
                                size="sm"
                                onClick={() =>
                                  window.open(bangumi.homepage!, '_blank')
                                }
                              >
                                <ExternalLink className="mr-2 h-3 w-3" />
                                Homepage
                              </Button>
                            </div>
                          )}
                        </Card>
                      ))}
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
