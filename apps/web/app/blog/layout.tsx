import { Toolbar } from '@konobangu/cms/components/toolbar';
import type { ReactNode } from 'react';

type BlogLayoutProps = {
  children: ReactNode;
};

const BlogLayout = ({ children }: BlogLayoutProps) => (
  <>
    {children}
    <Toolbar />
  </>
);

export default BlogLayout;
