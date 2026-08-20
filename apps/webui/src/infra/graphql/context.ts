import type {
  SecuritydeptInjectorTrait as Injector,
  SecuritydeptProvider as Provider,
} from "@securitydept/client";
import { APP_LIFETIME, type AppLifetime } from "@/app/lifetime/scope";
import { GraphQLService } from "./graphql.service";

export function provideGraphql(): Provider[] {
  return [
    {
      provide: GraphQLService,
      useFactory: (lifetime: AppLifetime) =>
        GraphQLService.createOwned(lifetime),
      deps: [APP_LIFETIME],
    },
  ];
}

export interface GraphQLContext {
  graphqlService: GraphQLService;
}

export function graphqlContextFromInjector(injector: Injector): GraphQLContext {
  return {
    graphqlService: injector.get(GraphQLService),
  };
}
