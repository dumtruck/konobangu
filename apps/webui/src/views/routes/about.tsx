import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/about')({
  component: About,
});

function About() {
  return <div class="p-2">Hello from About!</div>;
}
