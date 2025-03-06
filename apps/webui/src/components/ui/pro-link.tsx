import { type LinkComponent, createLink } from '@tanstack/solid-router';
import {
  type Component,
  type ComponentProps,
  type JSX,
  Show,
  splitProps,
} from 'solid-js';

type BasicLinkProps = JSX.IntrinsicElements['a'];

const BasicLinkComponent: Component<BasicLinkProps> = (props) => (
  <a {...props}>{props.children}</a>
);

const CreatedLinkComponent = createLink(BasicLinkComponent);

export const ProLink: LinkComponent<typeof BasicLinkComponent> = (props) => {
  const [local, other] = splitProps(props, ['href']);
  return (
    <Show
      when={!props.href}
      fallback={<BasicLinkComponent {...(other as any)} href={local.href} />}
    >
      <CreatedLinkComponent preload={'intent'} {...(other as typeof props)} />
    </Show>
  );
};

export type ProLinkProps = ComponentProps<typeof ProLink>;
