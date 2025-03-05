import { type LinkComponent, createLink } from '@tanstack/solid-router';
import { type Component, type ComponentProps, type JSX, Show } from 'solid-js';

type BasicLinkProps = JSX.IntrinsicElements['a'];

const BasicLinkComponent: Component<BasicLinkProps> = (props) => (
  <a {...props}>{props.children}</a>
);

const CreatedLinkComponent = createLink(BasicLinkComponent);

export const ProLink: LinkComponent<typeof BasicLinkComponent> = (props) => {
  return (
    <Show
      when={props.href}
      fallback={<CreatedLinkComponent preload={'intent'} {...props} />}
    >
      <BasicLinkComponent {...(props as any)} />
    </Show>
  );
};

export type ProLinkProps = ComponentProps<typeof ProLink>;
