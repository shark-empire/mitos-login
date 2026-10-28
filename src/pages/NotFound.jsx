import { Button, PageHeader } from '../components/ui';
import { usePageTitle } from '../hooks/usePageTitle';

export default function NotFound() {
  usePageTitle('Page not found');
  return (
    <div className="container page page--narrow">
      <PageHeader title="We can’t find that page" lead="The link may be old or mistyped." />
      <div className="status__actions status__actions--left">
        <Button to="/">Go to the home page</Button>
        <Button to="/data" variant="secondary">Buy data</Button>
      </div>
    </div>
  );
}
