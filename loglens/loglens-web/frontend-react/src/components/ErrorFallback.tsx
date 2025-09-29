import { FallbackProps } from 'react-error-boundary';
import { ExclamationTriangleIcon, ArrowPathIcon } from '@heroicons/react/24/outline';

function ErrorFallback({ error, resetErrorBoundary }: FallbackProps) {
  const isDev = (import.meta as any).env?.DEV || false;

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
      <div className="max-w-md w-full mx-auto">
        <div className="card">
          <div className="card-body text-center">
            <div className="mx-auto flex items-center justify-center h-12 w-12 rounded-full bg-error-100 dark:bg-error-900/20">
              <ExclamationTriangleIcon
                className="h-6 w-6 text-error-600 dark:text-error-400"
                aria-hidden="true"
              />
            </div>

            <h2 className="mt-4 text-lg font-semibold text-gray-900 dark:text-gray-100">
              Something went wrong
            </h2>

            <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
              We're sorry, but an unexpected error occurred. Please try refreshing the page or contact support if the problem persists.
            </p>

            {isDev && (
              <details className="mt-4 text-left">
                <summary className="cursor-pointer text-sm font-medium text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-gray-100">
                  Error Details (Development)
                </summary>
                <pre className="mt-2 text-xs bg-gray-100 dark:bg-gray-800 p-3 rounded-md overflow-auto max-h-40 text-error-600 dark:text-error-400">
                  {error.message}
                  {error.stack && `\n\n${error.stack}`}
                </pre>
              </details>
            )}

            <div className="mt-6">
              <button
                type="button"
                onClick={resetErrorBoundary}
                className="btn-primary"
              >
                <ArrowPathIcon className="h-4 w-4 mr-2" aria-hidden="true" />
                Try again
              </button>
            </div>

            <p className="mt-4 text-xs text-gray-500 dark:text-gray-400">
              Error ID: {Date.now().toString(36)}
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}

export default ErrorFallback;