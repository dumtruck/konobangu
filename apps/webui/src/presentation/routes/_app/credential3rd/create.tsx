import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { FormFieldErrors } from '@/components/ui/form-field-errors';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useAppForm } from '@/components/ui/tanstack-form';
import { Textarea } from '@/components/ui/textarea';
import {
  type Credential3rdInsertDto,
  Credential3rdInsertSchema,
  INSERT_CREDENTIAL_3RD,
} from '@/domains/recorder/schema/credential3rd';
import { useInject } from '@/infra/di/inject';
import {
  Credential3rdTypeEnum,
  type InsertCredential3rdMutation,
} from '@/infra/graphql/gql/graphql';
import { PlatformService } from '@/infra/platform/platform.service';
import type { RouteStateDataOption } from '@/infra/routes/traits';
import { useMutation } from '@apollo/client';
import { createFileRoute, useNavigate } from '@tanstack/react-router';
import { Loader2, Save } from 'lucide-react';
import { toast } from 'sonner';

export const Route = createFileRoute('/_app/credential3rd/create')({
  component: CredentialCreateRouteComponent,
  staticData: {
    breadcrumb: { label: 'Create' },
  } satisfies RouteStateDataOption,
});

function CredentialCreateRouteComponent() {
  const navigate = useNavigate();
  const platformService = useInject(PlatformService);

  const [insertCredential3rd, { loading }] = useMutation<
    InsertCredential3rdMutation['credential3rdCreateOne']
  >(INSERT_CREDENTIAL_3RD, {
    onCompleted(data) {
      toast.success('Credential created');
      navigate({
        to: '/credential3rd/detail/$id',
        params: { id: `${data.id}` },
      });
    },
    onError(error) {
      toast.error('Failed to create credential', {
        description: error.message,
      });
    },
  });

  const form = useAppForm({
    defaultValues: {
      credentialType: Credential3rdTypeEnum.Mikan,
      username: '',
      password: '',
      userAgent: '',
    } as Credential3rdInsertDto,
    validators: {
      onBlur: Credential3rdInsertSchema,
      onSubmit: Credential3rdInsertSchema,
    },
    onSubmit: async (form) => {
      if (form.value.credentialType === Credential3rdTypeEnum.Mikan) {
        await insertCredential3rd({
          variables: {
            data: form.value,
          },
        });
      }
    },
  });

  return (
    <div className="container mx-auto max-w-2xl py-6">
      <div className="mb-6 flex items-center gap-4">
        <div>
          <h1 className="font-bold text-2xl">Create third-party credential</h1>
          <p className="mt-1 text-muted-foreground">
            Add new third-party login credential
          </p>
        </div>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Credential information</CardTitle>
          <CardDescription className="mt-2">
            Please fill in the information of the third-party platform login
            credential for your subscriptions.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form
            onSubmit={(e) => {
              e.preventDefault();
              e.stopPropagation();
              form.handleSubmit();
            }}
            className="space-y-6"
          >
            <form.Field
              name="credentialType"
              validators={{
                onChange: ({ value }) => {
                  if (!value) {
                    return 'Please select the credential type';
                  }
                },
              }}
            >
              {(field) => (
                <div className="space-y-2">
                  <Label htmlFor={field.name}>Credential type *</Label>
                  <Select
                    value={field.state.value}
                    onValueChange={(value) =>
                      field.handleChange(value as Credential3rdTypeEnum)
                    }
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select credential type" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value={Credential3rdTypeEnum.Mikan}>
                        Mikan
                      </SelectItem>
                    </SelectContent>
                  </Select>
                  {field.state.meta.errors && (
                    <p className="text-destructive text-sm">
                      {field.state.meta.errors[0]?.toString()}
                    </p>
                  )}
                </div>
              )}
            </form.Field>
            <form.Field name="username">
              {(field) => (
                <div className="space-y-2">
                  <Label htmlFor={field.name}>Username *</Label>
                  <Input
                    id={field.name}
                    name={field.name}
                    value={field.state.value}
                    onBlur={field.handleBlur}
                    onChange={(e) => field.handleChange(e.target.value)}
                    placeholder="Please enter username"
                    autoComplete="off"
                  />
                  {field.state.meta.errors && (
                    <FormFieldErrors errors={field.state.meta.errors} />
                  )}
                </div>
              )}
            </form.Field>

            {/* 密码 */}
            <form.Field name="password">
              {(field) => (
                <div className="space-y-2">
                  <Label htmlFor={field.name}>Password *</Label>
                  <Input
                    id={field.name}
                    name={field.name}
                    type="password"
                    value={field.state.value}
                    onBlur={field.handleBlur}
                    onChange={(e) => field.handleChange(e.target.value)}
                    placeholder="Please enter password"
                    autoComplete="off"
                  />
                  {field.state.meta.errors && (
                    <FormFieldErrors errors={field.state.meta.errors} />
                  )}
                </div>
              )}
            </form.Field>

            {/* User Agent */}
            <form.Field name="userAgent">
              {(field) => (
                <div className="space-y-2">
                  <Label htmlFor={field.name}>User Agent</Label>
                  <Textarea
                    id={field.name}
                    name={field.name}
                    value={field.state.value || ''}
                    onBlur={field.handleBlur}
                    onChange={(e) => field.handleChange(e.target.value)}
                    rows={3}
                    className="resize-none"
                    autoComplete="off"
                    placeholder="Please enter User Agent (optional), leave it blank to use
                    the default value"
                  />
                  <p className="text-muted-foreground text-sm">
                    Current user agent: {platformService.userAgent}
                  </p>
                  {field.state.meta.errors && (
                    <FormFieldErrors errors={field.state.meta.errors} />
                  )}
                </div>
              )}
            </form.Field>

            <div className="flex gap-3 pt-4">
              <form.Subscribe
                selector={(state) => [state.canSubmit, state.isSubmitting]}
              >
                {([canSubmit, isSubmitting]) => (
                  <Button
                    type="submit"
                    disabled={!canSubmit || loading}
                    className="flex-1"
                  >
                    {loading || isSubmitting ? (
                      <>
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        Creating...
                      </>
                    ) : (
                      <>
                        <Save className="mr-2 h-4 w-4" />
                        Create credential
                      </>
                    )}
                  </Button>
                )}
              </form.Subscribe>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
