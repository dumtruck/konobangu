import { ApolloClient, createHttpLink, InMemoryCache } from '@apollo/client';
import { setContext } from '@apollo/client/link/context';
import { Injectable, inject } from '@outposts/injection-js';
import { firstValueFrom } from 'rxjs';
import { AUTH_PROVIDER } from '@/infra/auth/auth.provider';

@Injectable()
export class GraphQLService {
  private authProvider = inject(AUTH_PROVIDER);

  private apiLink = createHttpLink({
    uri: '/api/graphql',
  });

  private authLink = setContext(async (_, { headers }) => {
    const authHeaders = await firstValueFrom(
      this.authProvider.getAuthHeaders()
    );
    return { headers: { ...headers, ...authHeaders } };
  });

  _apollo = new ApolloClient({
    link: this.authLink.concat(this.apiLink),
    cache: new InMemoryCache(),
    defaultOptions: {
      watchQuery: {
        fetchPolicy: 'cache-and-network',
        nextFetchPolicy: 'network-only',
        errorPolicy: 'all',
        refetchWritePolicy: 'overwrite',
        initialFetchPolicy: 'cache-and-network',
      },
      query: {
        fetchPolicy: 'network-only',
        errorPolicy: 'all',
      },
    },
    connectToDevTools: process.env.NODE_ENV === 'development',
  });

  query = this._apollo.query;
  mutate = this._apollo.mutate;
  watchQuery = this._apollo.watchQuery;
}
