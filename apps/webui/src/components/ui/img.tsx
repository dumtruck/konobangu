import { type ComponentProps, useMemo, useState } from "react";

export type ImgProps = Omit<ComponentProps<"img">, "alt"> &
  Required<Pick<ComponentProps<"img">, "alt">> & {
    optimize?: boolean;
  };

const LEGACY_IMAGE_REGEX = /\.(jpg|jpeg|png|gif|svg)$/;

export const Img = (props: ImgProps) => {
  const src = props.src;

  const isLegacy = useMemo(() => src?.match(LEGACY_IMAGE_REGEX), [src]);
  const [isError, setIsError] = useState(false);

  if (!src) {
    // biome-ignore lint/nursery/noImgElement: <explanation>
    return <img {...props} alt={props.alt} />;
  }

  return (
    <picture {...props}>
      {isLegacy && !isError && (
        <>
          <source
            srcSet={src.replace(LEGACY_IMAGE_REGEX, ".webp")}
            type="image/webp"
          />
          <source
            srcSet={src.replace(LEGACY_IMAGE_REGEX, ".avif")}
            type="image/avif"
          />
        </>
      )}
      <img {...props} alt={props.alt} onError={() => setIsError(true)} />
    </picture>
  );
};
