import { PlatformService } from "./platform.service";
export const providePlatform = () => [
  {
    provide: PlatformService,
    useFactory: () => new PlatformService(),
    deps: [],
  },
];
