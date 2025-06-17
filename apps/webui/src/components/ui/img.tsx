import { type ComponentProps } from "react";

export type ImgProps = Omit<ComponentProps<"img">, "alt"> &
  Required<Pick<ComponentProps<"img">, "alt">> & {
    optimize?: boolean;
  };

// biome-ignore lint/correctness/noUnusedVariables: <explanation>
const LEGACY_IMAGE_REGEX = /\.(jpg|jpeg|png|gif|svg)$/;

export const Img = (props: ImgProps) => {
  const src = props.src;

  if (!src) {
    // biome-ignore lint/nursery/noImgElement: <explanation>
    return <img {...props} alt={props.alt} />;
  }

  return (
    <picture {...props}>
      <img {...props} alt={props.alt} />
    </picture>
  );
};
