import { DocumentTextIcon, ArrowDownIcon as DownloadIcon, ShareIcon } from '@heroicons/react/24/outline';
import { Analysis } from '@/types';
import { useState } from 'react';

interface ExportOptionsProps {
  analysis: Analysis | null;
  loading: boolean;
}

export function ExportOptions({ analysis, loading }: ExportOptionsProps) {
  const [exporting, setExporting] = useState<string | null>(null);
  const [exportError, setExportError] = useState<string | null>(null);

  const handleExport = async (format: 'json' | 'html' | 'md' | 'csv' | 'pdf') => {
    if (!analysis) return;

    setExporting(format);
    setExportError(null);

    try {
      const response = await fetch(`/api/projects/${analysis.project_id}/analyses/${analysis.id}/export/${format}`);
      
      if (!response.ok) {
        const errorText = await response.text();
        console.error('Export failed:', response.status, errorText);
        
        // Handle PDF specific errors
        if (format === 'pdf' && response.status === 500) {
          setExportError('PDF export failed. The server may be missing required dependencies. Try HTML export instead.');
        } else {
          setExportError(`Export failed (${response.status}): ${errorText || 'Unknown error'}`);
        }
        return;
      }

      const blob = await response.blob();
      
      // Check if we got an error page instead of the expected format
      if (format === 'pdf' && blob.type === 'text/html') {
        const text = await blob.text();
        if (text.includes('PDF Export Failed')) {
          setExportError('PDF export failed. The server may be missing wkhtmltopdf. Try HTML export instead.');
          return;
        }
      }

      // Create download link
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `loglens_analysis_${analysis.id}.${format}`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
    } catch (error) {
      console.error('Export failed:', error);
      setExportError(`Export failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setExporting(null);
    }
  };

  const handleShare = async () => {
    if (!analysis) return;

    const shareUrl = `${window.location.origin}/analysis/${analysis.id}`;
    
    if (navigator.share) {
      try {
        await navigator.share({
          title: `LogLens Analysis #${analysis.id}`,
          text: 'Check out this log analysis from LogLens',
          url: shareUrl,
        });
      } catch (error) {
        // Fallback to copying to clipboard
        await navigator.clipboard.writeText(shareUrl);
        alert('Analysis link copied to clipboard!');
      }
    } else {
      // Fallback to copying to clipboard
      await navigator.clipboard.writeText(shareUrl);
      alert('Analysis link copied to clipboard!');
    }
  };

  if (loading) {
    return (
      <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6 mb-6">
        <div className="animate-pulse">
          <div className="h-8 bg-gray-200 dark:bg-gray-700 rounded mb-4"></div>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-4">
            <div className="h-12 bg-gray-200 dark:bg-gray-700 rounded"></div>
            <div className="h-12 bg-gray-200 dark:bg-gray-700 rounded"></div>
            <div className="h-12 bg-gray-200 dark:bg-gray-700 rounded"></div>
            <div className="h-12 bg-gray-200 dark:bg-gray-700 rounded"></div>
            <div className="h-12 bg-gray-200 dark:bg-gray-700 rounded"></div>
          </div>
        </div>
      </div>
    );
  }

  if (!analysis) {
    return (
      <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6 mb-6">
        <div className="flex items-center space-x-2 mb-4">
          <DocumentTextIcon className="h-5 w-5 text-gray-400" />
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            Export Options
          </h2>
        </div>
        <p className="text-gray-500 dark:text-gray-400">
          No analysis available for export.
        </p>
      </div>
    );
  }

  const exportFormats = [
    {
      key: 'json',
      name: 'JSON',
      description: 'Machine-readable format for further processing',
      extension: 'json',
      icon: 'DATA'
    },
    {
      key: 'html',
      name: 'HTML Report',
      description: 'Formatted report with visualizations',
      extension: 'html',
      icon: 'WEB'
    },
    {
      key: 'md',
      name: 'Markdown',
      description: 'Documentation-friendly format',
      extension: 'md',
      icon: 'DOC'
    },
    {
      key: 'csv',
      name: 'CSV Data',
      description: 'Spreadsheet-compatible data format',
      extension: 'csv',
      icon: 'SHEET'
    },
    {
      key: 'pdf',
      name: 'PDF Report',
      description: 'Print-ready formatted document',
      extension: 'pdf',
      icon: 'DOC'
    }
  ];

  return (
    <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6 mb-6">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center space-x-2">
          <DocumentTextIcon className="h-5 w-5 text-blue-600" />
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            Export Options
          </h2>
        </div>
        <button
          onClick={handleShare}
          disabled={exporting !== null}
          className="inline-flex items-center px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          <ShareIcon className="h-4 w-4 mr-2" />
          Share Analysis
        </button>
      </div>

      {exportError && (
        <div className="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-700 rounded-lg">
          <p className="text-sm text-red-800 dark:text-red-200">
            {exportError}
          </p>
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-4">
        {exportFormats.map((format) => (
          <button
            key={format.key}
            onClick={() => handleExport(format.key as 'json' | 'html' | 'md' | 'csv' | 'pdf')}
            disabled={exporting === format.key}
            className="border border-gray-200 dark:border-gray-700 rounded-lg p-4 hover:border-blue-300 dark:hover:border-blue-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <div className="flex flex-col items-center text-center">
              <div className="text-2xl mb-2">{format.icon}</div>
              <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-1">
                {format.name}
              </h3>
              <p className="text-xs text-gray-500 dark:text-gray-400 mb-3">
                {format.description}
              </p>
              <div className="flex items-center space-x-1 text-xs text-blue-600 dark:text-blue-400">
                <DownloadIcon className="h-3 w-3" />
                <span>{format.extension.toUpperCase()}</span>
              </div>
            </div>
          </button>
        ))}
      </div>

      <div className="mt-6 p-4 bg-gray-50 dark:bg-gray-700 rounded-lg">
        <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-2">
          Export Information
        </h3>
        <div className="grid grid-cols-2 gap-4 text-sm">
          <div>
            <p className="text-gray-500 dark:text-gray-400">Analysis ID</p>
            <p className="font-mono text-gray-900 dark:text-gray-100">
              {analysis.id}
            </p>
          </div>
          <div>
            <p className="text-gray-500 dark:text-gray-400">Provider</p>
            <p className="text-gray-900 dark:text-gray-100">
              {analysis.ai_provider}
            </p>
          </div>
          <div>
            <p className="text-gray-500 dark:text-gray-400">Status</p>
            <p className="text-gray-900 dark:text-gray-100">
              {analysis.status}
            </p>
          </div>
          <div>
            <p className="text-gray-500 dark:text-gray-400">Created</p>
            <p className="text-gray-900 dark:text-gray-100">
              {new Date(analysis.created_at).toLocaleDateString()}
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}