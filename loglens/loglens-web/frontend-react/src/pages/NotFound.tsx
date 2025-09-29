import { Link } from 'react-router-dom';
import { HomeIcon } from '@heroicons/react/24/outline';

function NotFound() {
  return (
    <div className="min-h-96 flex items-center justify-center">
      <div className="text-center">
        <h1 className="text-6xl font-bold text-gray-300 dark:text-gray-600">404</h1>
        <h2 className="mt-4 text-2xl font-semibold text-gray-900 dark:text-white">
          Page not found
        </h2>
        <p className="mt-2 text-gray-600 dark:text-gray-400">
          The page you're looking for doesn't exist.
        </p>
        <div className="mt-6">
          <Link to="/dashboard" className="btn-primary">
            <HomeIcon className="h-4 w-4 mr-2" aria-hidden="true" />
            Go to Dashboard
          </Link>
        </div>
      </div>
    </div>
  );
}

export default NotFound;