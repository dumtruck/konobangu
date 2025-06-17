import type { ComponentProps } from "react";

export type ImgProps = Omit<ComponentProps<"img">, "alt"> &
  Required<Pick<ComponentProps<"img">, "alt">>;

export const Img = (props: ImgProps) => {
  // biome-ignore lint/nursery/noImgElement: <explanation>
  return <img {...props} alt={props.alt} />;
};
