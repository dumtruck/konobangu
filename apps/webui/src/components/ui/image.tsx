import type { ComponentProps } from 'react';

export type ImageProps = Omit<ComponentProps<'img'>, 'alt'> &
  Required<Pick<ComponentProps<'img'>, 'alt'>>;

export const Image = (props: ImageProps) => {
  // biome-ignore lint/nursery/noImgElement: <explanation>
  return <img {...props} alt={props.alt} />;
};
