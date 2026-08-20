import { useMutation, useQuery } from "@apollo/client/react";
import { createFileRoute } from "@tanstack/react-router";
import { type } from "arktype";
import { Save } from "lucide-react";
import { useCallback, useMemo } from "react";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { ContainerHeader } from "@/components/ui/container-header";
import { Cron } from "@/components/ui/cron";
import { CronMode } from "@/components/ui/cron/types";
import { DetailCardSkeleton } from "@/components/ui/detail-card-skeleton";
import { DetailEmptyView } from "@/components/ui/detail-empty-view";
import { FormFieldErrors } from "@/components/ui/form-field-errors";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  QueryErrorView,
  QueryPartialError,
} from "@/components/ui/query-error-view";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import { useAppForm } from "@/components/ui/tanstack-form";
import { GET_CRONS, UPDATE_CRONS } from "@/domains/recorder/schema/cron";
import {
  apolloErrorToMessage,
  getApolloQueryError,
} from "@/infra/errors/apollo";
import type { GetCronsQuery } from "@/infra/graphql/gql/graphql";
import type { RouteStateDataOption } from "@/infra/routes/traits";

export const Route = createFileRoute("/_app/tasks/cron/edit/$id")({
  component: CronEditRouteComponent,
  staticData: {
    breadcrumb: { label: "Edit" },
  } satisfies RouteStateDataOption,
});

type CronDetailDto = NonNullable<GetCronsQuery["cron"]["nodes"][0]> & {
  cronTimezone: string;
};

// The plan owns its schedule and whether it is active; execution parameters
// belong to the task runs it creates. The schema is the form's contract, and
// the nullable GraphQL input type only applies at the mutation boundary.
const CronUpdateFormSchema = type({
  cronExpr: "string>0",
  cronTimezone: "string>0",
  enabled: "boolean",
});

type CronUpdateForm = typeof CronUpdateFormSchema.infer;

function FormView({
  cron,
  onCompleted,
}: {
  cron: CronDetailDto;
  onCompleted: VoidFunction;
}) {
  const [updateCron, { loading: updating }] = useMutation(UPDATE_CRONS, {
    onCompleted,
    onError: (error) => {
      toast.error("Update cron task failed", {
        description: error.message,
      });
    },
  });

  // Initialize form data
  const defaultValues = useMemo(
    (): CronUpdateForm => ({
      cronExpr: cron.cronExpr,
      cronTimezone: cron.cronTimezone,
      enabled: cron.enabled,
    }),
    [cron],
  );

  const form = useAppForm({
    defaultValues,
    validators: {
      onChangeAsync: CronUpdateFormSchema,
      onChangeAsyncDebounceMs: 300,
      onSubmit: CronUpdateFormSchema,
    },
    onSubmit: async (submittedForm) => {
      const { cronExpr, cronTimezone, enabled } = submittedForm.value;

      await updateCron({
        variables: {
          data: {
            cronExpr,
            cronTimezone,
            enabled,
          },
          filter: {
            id: {
              eq: cron.id,
            },
          },
        },
      });
    },
  });

  return (
    <div className="container mx-auto max-w-4xl py-6">
      <ContainerHeader
        title="Edit cron task"
        description={`Edit cron task #${cron.id}`}
        defaultBackTo={`/tasks/cron/detail/${cron.id}`}
        actions={
          <Button onClick={() => form.handleSubmit()} disabled={updating}>
            <Save className="mr-2 h-4 w-4" />
            {updating ? "Saving..." : "Save"}
          </Button>
        }
      />

      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="text-base">Cron task information</CardTitle>
              <CardDescription className="mt-2">
                Edit cron task configuration
              </CardDescription>
            </div>
            <Badge variant={cron.enabled ? "default" : "secondary"}>
              {cron.enabled ? "Enabled" : "Disabled"}
            </Badge>
          </div>
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
            {/* Cron expression */}
            <form.Field name="cronExpr">
              {(field) => (
                <form.Subscribe selector={(state) => state.values.cronTimezone}>
                  {(cronTimezone) => (
                    <div className="space-y-2">
                      <Cron
                        defaultExpanded={false}
                        mode={CronMode.BothExpandable}
                        value={field.state.value}
                        onChange={(value) => field.handleChange(value)}
                        timezone={cronTimezone}
                        showPreview={true}
                        showDescription={true}
                        withCard={false}
                        placeholder="0 0 * * * *"
                        titleClassName="font-normal"
                      />
                    </div>
                  )}
                </form.Subscribe>
              )}
            </form.Field>
            {/* Timezone */}
            <form.Field
              name="cronTimezone"
              listeners={{
                onChangeDebounceMs: 300,
              }}
            >
              {(field) => (
                <div className="space-y-2">
                  <Label htmlFor={field.name}>Timezone *</Label>
                  <Input
                    id={field.name}
                    name={field.name}
                    value={field.state.value}
                    onBlur={field.handleBlur}
                    onChange={(e) => field.handleChange(e.target.value)}
                    placeholder="Asia/Shanghai"
                    autoComplete="off"
                  />
                  {field.state.meta.errors && (
                    <FormFieldErrors
                      errors={field.state.meta.errors}
                      isDirty={field.state.meta.isDirty}
                      submissionAttempts={form.state.submissionAttempts}
                    />
                  )}
                </div>
              )}
            </form.Field>
            {/* Enabled */}
            <form.Field name="enabled">
              {(field) => (
                <div className="flex items-center justify-between">
                  <div className="space-y-0.5">
                    <Label htmlFor={field.name}>Enabled</Label>
                    <div className="text-muted-foreground text-sm">
                      Enable this cron task
                    </div>
                  </div>
                  <Switch
                    id={field.name}
                    checked={field.state.value}
                    onCheckedChange={(checked) => field.handleChange(checked)}
                  />
                </div>
              )}
            </form.Field>

            {/* Task detail */}
            {cron.subscriberTaskCron && (
              <>
                <Separator />
                <div className="space-y-2">
                  <Label className="font-medium text-base">Task detail</Label>
                  <div className="rounded-md bg-muted p-3">
                    <pre className="overflow-x-auto whitespace-pre-wrap text-sm">
                      <code>
                        {JSON.stringify(cron.subscriberTaskCron, null, 2)}
                      </code>
                    </pre>
                  </div>
                </div>
              </>
            )}
          </form>
        </CardContent>
      </Card>
    </div>
  );
}

function CronEditRouteComponent() {
  const { id } = Route.useParams();

  const { loading, error, data, refetch } = useQuery(GET_CRONS, {
    variables: {
      filter: {
        id: {
          eq: Number.parseInt(id, 10),
        },
      },
      pagination: {
        page: {
          page: 0,
          limit: 1,
        },
      },
      orderBy: {},
    },
  });

  const cron = data?.cron?.nodes?.[0];

  const onCompleted = useCallback(async () => {
    const refetchResult = await refetch();
    const _error = getApolloQueryError(refetchResult);
    if (_error) {
      toast.error("Update cron task failed", {
        description: apolloErrorToMessage(_error),
      });
    } else {
      toast.success("Update cron task success");
    }
  }, [refetch]);

  if (loading && !data) {
    return <DetailCardSkeleton />;
  }

  if (error && !data) {
    return <QueryErrorView message={error.message} />;
  }

  if (!cron) {
    return <DetailEmptyView message="Not found cron" />;
  }

  return (
    <>
      <QueryPartialError error={error} />
      <FormView cron={cron as CronDetailDto} onCompleted={onCompleted} />
    </>
  );
}
