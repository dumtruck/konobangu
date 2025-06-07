import { Button } from '@/components/ui/button';
import {
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { CHECK_CREDENTIAL_3RD_AVAILABLE } from '@/domains/recorder/schema/credential3rd';
import {
  apolloErrorToMessage,
  getApolloQueryError,
} from '@/infra/errors/apollo';
import type { CheckCredential3rdAvailableQuery } from '@/infra/graphql/gql/graphql';
import { useLazyQuery } from '@apollo/client';
import { CheckIcon, Loader2, XIcon } from 'lucide-react';
import { memo, useCallback } from 'react';
import { toast } from 'sonner';

export interface Credential3rdCheckAvailableViewProps {
  id: number;
}

export const Credential3rdCheckAvailableView = memo(
  ({ id }: Credential3rdCheckAvailableViewProps) => {
    const [checkAvailable, { data, error, loading }] =
      useLazyQuery<CheckCredential3rdAvailableQuery>(
        CHECK_CREDENTIAL_3RD_AVAILABLE,
        {
          variables: { id },
        }
      );

    const handleCheckAvailable = useCallback(async () => {
      const checkResult = await checkAvailable();
      const error = getApolloQueryError(checkResult);
      console.error('error', error);
      if (error) {
        toast.error('Failed to check available', {
          description: apolloErrorToMessage(error),
        });
        return;
      }
      if (checkResult.data?.credential3rdCheckAvailable.available) {
        toast.success('Credential is available');
      } else {
        toast.error('Credential is not available');
      }
    }, [checkAvailable]);

    const available = data?.credential3rdCheckAvailable?.available;

    return (
      <div className="flex flex-col gap-2">
        <Button
          variant="outline"
          size="lg"
          onClick={handleCheckAvailable}
          disabled={loading}
        >
          <span> Check Available </span>
          {available === true && (
            <CheckIcon className="h-4 w-4 text-green-300" />
          )}
          {(available === false || !!error) && (
            <XIcon className="h-4 w-4 text-red-500" />
          )}
          {loading && <Loader2 className="h-4 w-4 animate-spin" />}
        </Button>
      </div>
    );
  }
);

export interface Credential3rdCheckAvailableViewDialogContentProps {
  id: number;
}

export const Credential3rdCheckAvailableViewDialogContent = memo(
  ({ id }: Credential3rdCheckAvailableViewDialogContentProps) => {
    return (
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Check Available</DialogTitle>
          <DialogDescription>
            Check if the credential is available.
          </DialogDescription>
        </DialogHeader>
        <Credential3rdCheckAvailableView id={id} />
      </DialogContent>
    );
  }
);
