import { useAuth } from '@/auth/hooks';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import type { RouteStateDataOption } from '@/traits/router';
import { createFileRoute } from '@tanstack/react-router';
import { useNavigate } from '@tanstack/react-router';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { toast } from 'sonner';

export const Route = createFileRoute('/_app/subscriptions/create')({
  component: SubscriptionCreateRouteComponent,
  staticData: {
    breadcrumb: { label: 'Create' },
  } satisfies RouteStateDataOption,
});

type SubscriptionFormValues = {
  displayName: string;
  sourceUrl: string;
  category: string;
  enabled: boolean;
};

function SubscriptionCreateRouteComponent() {
  const { userData } = useAuth();
  console.log(JSON.stringify(userData, null, 2));
  const [isSubmitting, setIsSubmitting] = useState(false);
  const navigate = useNavigate();
  const form = useForm<SubscriptionFormValues>({
    defaultValues: {
      displayName: '',
      sourceUrl: '',
      category: 'Mikan',
      enabled: true,
    },
  });

  const onSubmit = async (data: SubscriptionFormValues) => {
    try {
      setIsSubmitting(true);
      const requestData = {
        query: `
          mutation CreateSubscription($input: SubscriptionsInsertInput!) {
            subscriptionsCreateOne(data: $input) {
              id
              displayName
              sourceUrl
              enabled
              category
            }
          }
        `,
        variables: {
          input: {
            category: data.category,
            displayName: data.displayName,
            sourceUrl: data.sourceUrl,
            enabled: data.enabled,
          },
        },
      };

      const response = await fetch('/api/graphql', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(requestData),
      });

      const responseData = await response.json();
      if (responseData.errors) {
        throw new Error(
          responseData.errors[0]?.message || 'Failed to create subscription'
        );
      }

      toast.success('Subscription created successfully');
      navigate({ to: '/subscriptions/manage' });
    } catch (error) {
      console.error('Failed to create subscription:', error);
      toast.error(
        `Subscription creation failed: ${
          error instanceof Error ? error.message : 'Unknown error'
        }`
      );
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Create Anime Subscription</CardTitle>
        <CardDescription>Add a new anime subscription source</CardDescription>
      </CardHeader>
      <CardContent>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
            <FormField
              control={form.control}
              name="category"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Source Type</FormLabel>
                  <Select
                    disabled
                    value={field.value}
                    onValueChange={field.onChange}
                    defaultValue="Mikan"
                  >
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue placeholder="Select source type" />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      <SelectItem value="Mikan">Mikan</SelectItem>
                    </SelectContent>
                  </Select>
                  <FormDescription>
                    Currently only Mikan source is supported
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="displayName"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Display Name</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="Enter subscription display name"
                      {...field}
                    />
                  </FormControl>
                  <FormDescription>
                    Set an easily recognizable name for this subscription
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="sourceUrl"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Source URL</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="Enter subscription source URL"
                      {...field}
                    />
                  </FormControl>
                  <FormDescription>
                    Copy the RSS subscription link from the source website, e.g.
                    https://mikanani.me/RSS/Bangumi?bangumiId=3141&subgroupid=370
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="enabled"
              render={({ field }) => (
                <FormItem className="flex flex-row items-center justify-between rounded-lg border p-4">
                  <div className="space-y-0.5">
                    <FormLabel className="text-base">
                      Enable Subscription
                    </FormLabel>
                    <FormDescription>
                      Enable this subscription immediately after creation
                    </FormDescription>
                  </div>
                  <FormControl>
                    <Switch
                      checked={field.value}
                      onCheckedChange={field.onChange}
                    />
                  </FormControl>
                </FormItem>
              )}
            />
          </form>
        </Form>
      </CardContent>
      <CardFooter className="flex justify-between">
        <Button
          variant="outline"
          onClick={() => navigate({ to: '/subscriptions/manage' })}
        >
          Cancel
        </Button>
        <Button
          type="submit"
          onClick={form.handleSubmit(onSubmit)}
          disabled={isSubmitting}
        >
          {isSubmitting ? 'Creating...' : 'Create Subscription'}
        </Button>
      </CardFooter>
    </Card>
  );
}
