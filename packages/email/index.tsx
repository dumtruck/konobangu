import type { JSX } from 'react';

export interface SendOptions {
  from: string;
  to: string;
  subject: string;
  replyTo: string;
  react: JSX.Element;
}

export const konosend = {
  emails: {
    // biome-ignore lint/suspicious/useAwait: <explanation>
    send: async (_props: SendOptions) => {
      throw new Error('unimplemented');
    },
  },
};
