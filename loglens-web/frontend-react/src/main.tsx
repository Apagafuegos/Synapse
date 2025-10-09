import React from 'react';
import ReactDOM from 'react-dom/client';
import { QueryClient, QueryClientProvider } from 'react-query';
import { BrowserRouter } from 'react-router-dom';
import { ErrorBoundary } from 'react-error-boundary';

import App from './App';
import ErrorFallback from '@components/ErrorFallback';
import { ThemeProvider } from '@hooks/useTheme';
import { WebSocketProvider } from '@hooks/useWebSocket';
import { logger, logErrorBoundary } from './utils/logger';

import './styles/index.css';

// Initialize logging
logger.info('App', 'LogLens frontend starting up');

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: (failureCount, error: any) => {
        if (error?.status === 404) return false;
        return failureCount < 3;
      },
      staleTime: 5 * 60 * 1000, // 5 minutes
    },
  },
});

function Root() {
  return (
    <ErrorBoundary 
      FallbackComponent={ErrorFallback} 
      onError={(error, errorInfo) => {
        logErrorBoundary(error, errorInfo);
        console.error('React Error Boundary:', error, errorInfo);
      }}
    >
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <ThemeProvider>
            <WebSocketProvider>
              <App />
            </WebSocketProvider>
          </ThemeProvider>
        </BrowserRouter>
      </QueryClientProvider>
    </ErrorBoundary>
  );
}

logger.debug('App', 'Rendering React application');

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>
);