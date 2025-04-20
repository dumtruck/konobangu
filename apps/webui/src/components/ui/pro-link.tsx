import { cn } from '@/styles/utils';
import { type LinkComponent, createLink } from '@tanstack/react-router';
import type { AnchorHTMLAttributes, ComponentProps } from 'react';

export interface BasicLinkProps
  extends AnchorHTMLAttributes<HTMLAnchorElement> {}

const BasicLinkComponent = (props: ComponentProps<'a'>) => {
  return (
    <a
      {...props}
      className={cn('block px-3 py-2 text-blue-700', props.className)}
    />
  );
};

const CreatedLinkComponent = createLink(BasicLinkComponent);

export const ProLink: LinkComponent<typeof BasicLinkComponent> = (props) => {
  if (props.href) {
    return <BasicLinkComponent {...(props as any)} />;
  }
  return <CreatedLinkComponent preload={'intent'} {...props} />;
};

export type ProLinkProps = ComponentProps<typeof ProLink>;
