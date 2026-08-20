import { useSignal } from "@securitydept/client-react";
import {
  type ComponentProps,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { ApiTarget } from "@/app/config/context";
import { DOCUMENT } from "@/app/environment/traits";
import { RESOURCE_SERVICE } from "@/app/resource/resource-grants";
import { Button } from "@/components/ui/button";
import { useInject } from "@/infra/di/inject";

export function optimizedImageUrl(
  src: string | undefined,
  baseURI: string | undefined,
  optimize: "accept",
  staticBase: string,
) {
  if (!src || !baseURI) return src;
  try {
    const base = new URL(baseURI);
    const url = new URL(src, base);
    const target = new URL(staticBase, base);
    if (
      url.origin !== target.origin ||
      !url.pathname.startsWith(target.pathname) ||
      !["http:", "https:"].includes(url.protocol)
    )
      return src;
    url.searchParams.set("optimize", optimize);
    return url.toString();
  } catch {
    return src;
  }
}

export type ImgProps = Omit<ComponentProps<"img">, "alt"> &
  Required<Pick<ComponentProps<"img">, "alt">> & {
    optimize?: "accept";
  };

/**
 * Renders the resource domain's image binding. Which address to load, when to
 * derive it again and when it has failed belong to the binding; this
 * component connects it once its element is near the viewport and reports
 * what the native element did.
 */
function NativeImg({ src: propsSrc, optimize = "accept", ...props }: ImgProps) {
  const document = useInject(DOCUMENT);
  const api = useInject(ApiTarget);
  const resources = useInject(RESOURCE_SERVICE);
  const element = useRef<HTMLImageElement>(null);
  const src =
    optimizedImageUrl(
      propsSrc,
      document.baseURI,
      optimize,
      api.url("static/"),
    ) ?? "";
  const binding = useMemo(() => resources.bindImage(src), [resources, src]);
  const state = useSignal(binding.state);
  const [near, setNear] = useState(props.loading !== "lazy");
  useEffect(() => {
    if (!element.current || near || !binding.private) return;
    return resources.nearViewport(element.current, () => setNear(true));
  }, [near, binding, resources]);
  useEffect(() => {
    if (!near) return;
    binding.connect();
    return () => binding.disconnect();
  }, [binding, near]);
  return (
    <>
      <img
        key={state.generation}
        {...props}
        ref={element}
        alt={props.alt}
        src={state.status === "ready" && state.url ? state.url : undefined}
        crossOrigin={binding.private ? "anonymous" : props.crossOrigin}
        onLoad={(event) => {
          binding.loaded();
          props.onLoad?.(event);
        }}
        onError={(event) => {
          if (!binding.loadFailed()) props.onError?.(event);
        }}
      />
      {state.status === "failed" && (
        <span
          role="alert"
          className="inline-flex items-center gap-2 text-sm text-destructive"
        >
          Image unavailable.
          <Button
            type="button"
            size="sm"
            variant="outline"
            aria-label={`Retry image ${props.alt}`}
            onClick={() => binding.retry()}
          >
            Retry
          </Button>
        </span>
      )}
    </>
  );
}
export function Img(props: ImgProps) {
  return (
    <NativeImg key={JSON.stringify([props.src, props.loading])} {...props} />
  );
}
