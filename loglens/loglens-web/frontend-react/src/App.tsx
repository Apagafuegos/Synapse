import { Routes, Route, Navigate } from 'react-router-dom';
import { Suspense, lazy } from 'react';

import Layout from '@components/Layout';
import LoadingSpinner from '@components/LoadingSpinner';
import { useTheme } from '@hooks/useTheme';

// Lazy load pages for better performance
const Dashboard = lazy(() => import('@pages/Dashboard'));
const Projects = lazy(() => import('@pages/Projects'));
const ProjectDetail = lazy(() => import('@pages/ProjectDetail'));
const AnalysisDetail = lazy(() => import('@pages/AnalysisDetail'));
const Settings = lazy(() => import('@pages/Settings'));
const NotFound = lazy(() => import('@pages/NotFound'));

function App() {
  const { theme } = useTheme();

  return (
    <div className={theme === 'dark' ? 'dark' : ''}>
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 transition-colors">
        <Layout>
          <Suspense
            fallback={
              <div className="flex items-center justify-center min-h-96">
                <LoadingSpinner size="lg" />
              </div>
            }
          >
            <Routes>
              <Route path="/" element={<Navigate to="/dashboard" replace />} />
              <Route path="/dashboard" element={<Dashboard />} />
              <Route path="/projects" element={<Projects />} />
              <Route path="/projects/:id" element={<ProjectDetail />} />
              <Route path="/analysis/:id" element={<AnalysisDetail />} />
              <Route path="/settings" element={<Settings />} />
              <Route path="*" element={<NotFound />} />
            </Routes>
          </Suspense>
        </Layout>
      </div>
    </div>
  );
}

export default App;