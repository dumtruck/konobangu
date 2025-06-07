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
import { Label } from '@/components/ui/label';
import { QueryErrorView } from '@/components/ui/query-error-view';
import { Separator } from '@/components/ui/separator';
import { GET_CREDENTIAL_3RD_DETAIL } from '@/domains/recorder/schema/credential3rd';
import type { GetCredential3rdDetailQuery } from '@/infra/graphql/gql/graphql';
import type { RouteStateDataOption } from '@/infra/routes/traits';
import { useQuery } from '@apollo/client';
import {
  createFileRoute,
  useCanGoBack,
  useNavigate,
  useRouter,
} from '@tanstack/react-router';
import { format } from 'date-fns/format';
import { ArrowLeft, CheckIcon, Edit, Eye, EyeOff } from 'lucide-react';
import { useState } from 'react';
import { Credential3rdCheckAvailableViewDialogContent } from './-check-available';

export const Route = createFileRoute('/_app/credential3rd/detail/$id')({
  component: Credential3rdDetailRouteComponent,
  staticData: {
    breadcrumb: { label: 'Detail' },
  } satisfies RouteStateDataOption,
});

function Credential3rdDetailRouteComponent() {
  const { id } = Route.useParams();
  const navigate = useNavigate();
  const router = useRouter();
  const canGoBack = useCanGoBack();

  const [showPassword, setShowPassword] = useState(false);

  const handleBack = () => {
    if (canGoBack) {
      router.history.back();
    } else {
      navigate({
        to: '/credential3rd/manage',
      });
    }
  };

  const { loading, error, data } = useQuery<GetCredential3rdDetailQuery>(
    GET_CREDENTIAL_3RD_DETAIL,
    {
      variables: {
        id: Number.parseInt(id),
      },
    }
  );

  const handleEnterEditMode = () => {
    navigate({
      to: '/credential3rd/edit/$id',
      params: {
        id,
      },
    });
  };

  const togglePasswordVisibility = () => {
    setShowPassword((prev) => !prev);
  };

  const credential = data?.credential3rd?.nodes?.[0];

  if (loading) {
    return <DetailCardSkeleton />;
  }

  if (error) {
    return <QueryErrorView message={error.message} />;
  }

  if (!credential) {
    return <DetailEmptyView message="Not found certain credential" />;
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
            <h1 className="font-bold text-2xl">Credential detail</h1>
            <p className="mt-1 text-muted-foreground">
              View credential #{credential.id}
            </p>
          </div>
        </div>

        <div className="flex gap-2">
          <Button onClick={handleEnterEditMode}>
            <Edit className="mr-2 h-4 w-4" />
            Edit
          </Button>
        </div>
      </div>

      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Credential information</CardTitle>
              <CardDescription className="mt-2">
                View credential detail
              </CardDescription>
            </div>
            <div className="flex items-center gap-2">
              <Dialog>
                <DialogTrigger asChild>
                  <Button variant="outline" size="sm">
                    <CheckIcon className="h-4 w-4" />
                    Check Available
                  </Button>
                </DialogTrigger>
                <Credential3rdCheckAvailableViewDialogContent
                  id={credential.id}
                />
              </Dialog>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="space-y-6">
            <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
              <div className="space-y-2">
                <Label className="font-medium text-sm">ID</Label>
                <div className="rounded-md bg-muted p-3">
                  <code className="text-sm">{credential.id}</code>
                </div>
              </div>

              <div className="space-y-2">
                <Label className="font-medium text-sm">Credential type</Label>
                <div className="rounded-md bg-muted p-3">
                  <Badge variant="secondary">{credential.credentialType}</Badge>
                </div>
              </div>

              <div className="space-y-2">
                <Label className="font-medium text-sm">Username</Label>
                <div className="rounded-md bg-muted p-3">
                  <span className="text-sm">
                    {credential.username || 'Not set'}
                  </span>
                </div>
              </div>

              <div className="space-y-2">
                <Label className="font-medium text-sm">Password</Label>
                <div className="flex items-center justify-between rounded-md bg-muted p-3">
                  <span className="font-mono text-sm">
                    {showPassword ? credential.password || '-' : '••••••••'}
                  </span>
                  {credential.password && (
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-6 w-6 p-0"
                      onClick={togglePasswordVisibility}
                    >
                      {showPassword ? (
                        <EyeOff className="h-3 w-3" />
                      ) : (
                        <Eye className="h-3 w-3" />
                      )}
                    </Button>
                  )}
                </div>
              </div>
            </div>

            <div className="space-y-2">
              <Label className="font-medium text-sm">User Agent</Label>
              <div className="rounded-md bg-muted p-3">
                <span className="break-all text-sm">
                  {credential.userAgent || '-'}
                </span>
              </div>
            </div>

            <Separator />

            <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
              <div className="space-y-2">
                <Label className="font-medium text-sm">Created at</Label>
                <div className="rounded-md bg-muted p-3">
                  <span className="text-sm">
                    {format(
                      new Date(credential.createdAt),
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
                      new Date(credential.updatedAt),
                      'yyyy-MM-dd HH:mm:ss'
                    )}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
