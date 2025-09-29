import { useState, useEffect, useMemo, useCallback } from 'react';
import { FixedSizeList as List } from 'react-window';
import { clsx } from 'clsx';
import { MagnifyingGlassIcon, FunnelIcon } from '@heroicons/react/24/outline';

// Type for react-window item data
type ItemData = {
  lines: LogEntry[];
  searchTerm: string;
  levelFilter: LogLevel | 'ALL';
};

import { wasmService, logUtils } from '@services/wasmService';
import type { LogLevel, LogEntry } from '@/types';

interface LogViewerProps {
  content: string;
  className?: string;
  height?: number;
  onAnalysisRequest?: (content: string) => void;
}

interface LogLineItemProps {
  index: number;
  style: React.CSSProperties;
  data: ItemData;
}

function LogLineItem({ index, style, data }: LogLineItemProps) {
  const { lines, searchTerm } = data;
  const line = lines[index];

  if (!line) return null;

  const shouldHighlight = searchTerm && line.message.toLowerCase().includes(searchTerm.toLowerCase());
  const levelClass = logUtils.getLogLevelClass(line.level);

  const highlightText = (text: string, term: string) => {
    if (!term) return text;
    const regex = new RegExp(`(${term})`, 'gi');
    return text.split(regex).map((part, i) =>
      regex.test(part) ? (
        <mark key={i} className="bg-yellow-200 dark:bg-yellow-800 px-1 rounded">
          {part}
        </mark>
      ) : (
        part
      )
    );
  };

  return (
    <div
      style={style}
      className={clsx(
        'flex items-start space-x-4 px-4 py-2 text-sm font-mono border-b border-gray-100 dark:border-gray-700',
        shouldHighlight && 'bg-yellow-50 dark:bg-yellow-900/20',
        levelClass
      )}
    >
      <span className="text-gray-500 dark:text-gray-400 w-12 flex-shrink-0 text-right">
        {line.line_number}
      </span>

      {line.timestamp && (
        <span className="text-gray-600 dark:text-gray-300 w-40 flex-shrink-0">
          {new Date(line.timestamp).toLocaleString()}
        </span>
      )}

      {line.level && (
        <span
          className={clsx(
            'px-2 py-1 text-xs font-semibold rounded uppercase w-16 text-center flex-shrink-0',
            {
              'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-300': line.level === 'ERROR',
              'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-300': line.level === 'WARN',
              'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300': line.level === 'INFO',
              'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300': line.level === 'DEBUG',
            }
          )}
        >
          {line.level}
        </span>
      )}

      <span className="flex-1 break-all">
        {highlightText(line.message, searchTerm)}
      </span>
    </div>
  );
}

export default function LogViewer({ content, className, height = 600, onAnalysisRequest }: LogViewerProps) {
  const [searchTerm, setSearchTerm] = useState('');
  const [levelFilter, setLevelFilter] = useState<LogLevel | 'ALL'>('ALL');
  const [parsedLogs, setParsedLogs] = useState<LogEntry[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [stats, setStats] = useState<Record<string, number>>({});

  // Parse logs with WASM when content changes
  useEffect(() => {
    if (!content.trim()) {
      setParsedLogs([]);
      return;
    }

    const parseContent = async () => {
      setIsLoading(true);
      setError(null);

      try {
        console.log('[LogViewer] Parsing content with WASM...');
        const startTime = performance.now();

        // Parse logs using WASM
        const parseResult = await wasmService.parseLogPreview(content, 10000);

        // Count log levels
        const levelCounts = await wasmService.countLogLevels(content);
        setStats(levelCounts);

        // Convert to LogEntry format
        const logEntries: LogEntry[] = parseResult.lines_by_level.map((line) => ({
          line_number: line.line_number,
          timestamp: line.timestamp || undefined,
          level: (line.level as LogLevel) || undefined,
          message: line.message,
          raw_line: line.message, // For now, use message as raw line
          is_truncated: line.is_truncated,
        }));

        setParsedLogs(logEntries);

        const endTime = performance.now();
        console.log(`[LogViewer] Parsed ${logEntries.length} lines in ${(endTime - startTime).toFixed(2)}ms`);
      } catch (err) {
        console.error('[LogViewer] Failed to parse logs:', err);
        setError(err instanceof Error ? err.message : 'Failed to parse logs');
      } finally {
        setIsLoading(false);
      }
    };

    parseContent();
  }, [content]);

  // Filter logs based on search term and level filter
  const filteredLogs = useMemo(() => {
    let filtered = parsedLogs;

    // Filter by level
    if (levelFilter !== 'ALL') {
      filtered = filtered.filter((log) => log.level === levelFilter);
    }

    // Filter by search term
    if (searchTerm.trim()) {
      const term = searchTerm.toLowerCase();
      filtered = filtered.filter((log) =>
        log.message.toLowerCase().includes(term) ||
        log.raw_line.toLowerCase().includes(term)
      );
    }

    return filtered;
  }, [parsedLogs, levelFilter, searchTerm]);

  const handleSearch = useCallback(async (term: string) => {
    if (!term.trim()) {
      setSearchTerm('');
      return;
    }

    try {
      setSearchTerm(term);
      // Use WASM search for better performance on large logs
      const matches = await wasmService.searchLogs(content, term, false);
      console.log(`[LogViewer] Found ${matches.length} matches for "${term}"`);
    } catch (err) {
      console.error('[LogViewer] Search failed:', err);
    }
  }, [content]);

  const handleAnalyzeRequest = useCallback(() => {
    if (onAnalysisRequest && content) {
      onAnalysisRequest(levelFilter !== 'ALL' ? filteredLogs.map(l => l.raw_line).join('\n') : content);
    }
  }, [onAnalysisRequest, content, levelFilter, filteredLogs]);

  if (isLoading) {
    return (
      <div className={clsx('flex items-center justify-center', className)} style={{ height }}>
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600 mx-auto"></div>
          <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">Parsing logs...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={clsx('flex items-center justify-center', className)} style={{ height }}>
        <div className="text-center text-error-600 dark:text-error-400">
          <p className="font-medium">Failed to parse logs</p>
          <p className="text-sm mt-1">{error}</p>
        </div>
      </div>
    );
  }

  return (
    <div className={clsx('border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden', className)}>
      {/* Header with controls */}
      <div className="bg-gray-50 dark:bg-gray-800 p-4 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100">
            Log Viewer ({filteredLogs.length.toLocaleString()} lines)
          </h3>

          {onAnalysisRequest && (
            <button
              onClick={handleAnalyzeRequest}
              className="px-4 py-2 bg-primary-600 text-white rounded-md hover:bg-primary-700 transition-colors"
            >
              Analyze with AI
            </button>
          )}
        </div>

        <div className="flex items-center space-x-4">
          {/* Search */}
          <div className="flex-1 relative">
            <MagnifyingGlassIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
            <input
              type="text"
              placeholder="Search logs..."
              value={searchTerm}
              onChange={(e) => handleSearch(e.target.value)}
              className="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
            />
          </div>

          {/* Level filter */}
          <div className="relative">
            <FunnelIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
            <select
              value={levelFilter}
              onChange={(e) => setLevelFilter(e.target.value as LogLevel | 'ALL')}
              className="pl-10 pr-8 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
            >
              <option value="ALL">All Levels</option>
              <option value="ERROR">Error ({stats.error || 0})</option>
              <option value="WARN">Warning ({stats.warning || 0})</option>
              <option value="INFO">Info ({stats.info || 0})</option>
              <option value="DEBUG">Debug ({stats.debug || 0})</option>
            </select>
          </div>
        </div>

        {/* Stats */}
        <div className="flex items-center space-x-6 mt-3 text-sm text-gray-600 dark:text-gray-400">
          <span className="flex items-center">
            <span className="w-2 h-2 bg-red-500 rounded-full mr-2"></span>
            {stats.error || 0} Errors
          </span>
          <span className="flex items-center">
            <span className="w-2 h-2 bg-yellow-500 rounded-full mr-2"></span>
            {stats.warning || 0} Warnings
          </span>
          <span className="flex items-center">
            <span className="w-2 h-2 bg-blue-500 rounded-full mr-2"></span>
            {stats.info || 0} Info
          </span>
          <span className="flex items-center">
            <span className="w-2 h-2 bg-gray-500 rounded-full mr-2"></span>
            {stats.debug || 0} Debug
          </span>
        </div>
      </div>

      {/* Log content */}
      <div style={{ height: height - 140 }}>
        {filteredLogs.length > 0 ? (
          <List
            height={height - 140}
            width="100%"
            itemCount={filteredLogs.length}
            itemSize={60}
            itemData={{
              lines: filteredLogs,
              searchTerm,
              levelFilter,
            }}
          >
            {LogLineItem}
          </List>
        ) : (
          <div className="flex items-center justify-center h-full text-gray-500 dark:text-gray-400">
            <p>No logs match the current filters</p>
          </div>
        )}
      </div>
    </div>
  );
}