import { useState, useCallback, useRef } from 'react';
import { CloudArrowUpIcon, DocumentTextIcon } from '@heroicons/react/24/outline';
import { clsx } from 'clsx';

interface FileUploadProps {
  onFileSelect: (file: File) => void;
  onFileUpload?: (file: File) => Promise<void>;
  isUploading?: boolean;
  error?: string | null;
  acceptedTypes?: string[];
  maxSizeBytes?: number;
  className?: string;
}

export default function FileUpload({
  onFileSelect,
  onFileUpload,
  isUploading = false,
  error = null,
  acceptedTypes = ['.log', '.txt', '.json'],
  maxSizeBytes = 100 * 1024 * 1024, // 100MB default
  className,
}: FileUploadProps) {
  const [isDragActive, setIsDragActive] = useState(false);
  const [uploadProgress, setUploadProgress] = useState(0);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const validateFile = useCallback((file: File) => {
    // Check file size
    if (file.size > maxSizeBytes) {
      throw new Error(`File size exceeds ${(maxSizeBytes / 1024 / 1024).toFixed(1)}MB limit`);
    }

    // Check file type if specified
    if (acceptedTypes.length > 0) {
      const fileExtension = '.' + file.name.split('.').pop()?.toLowerCase();
      const isValidType = acceptedTypes.some(type =>
        type.toLowerCase() === fileExtension ||
        file.type.includes(type.replace('.', ''))
      );

      if (!isValidType) {
        throw new Error(`File type not supported. Accepted types: ${acceptedTypes.join(', ')}`);
      }
    }

    return true;
  }, [acceptedTypes, maxSizeBytes]);

  const handleFileSelect = useCallback(async (file: File) => {
    try {
      validateFile(file);
      onFileSelect(file);

      if (onFileUpload) {
        setUploadProgress(0);
        await onFileUpload(file);
        setUploadProgress(100);
      }
    } catch (err) {
      console.error('File upload error:', err);
      // Error should be handled by parent component
    }
  }, [validateFile, onFileSelect, onFileUpload]);

  const handleDragEnter = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragActive(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragActive(false);
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragActive(false);

    const files = Array.from(e.dataTransfer.files);
    if (files.length > 0) {
      handleFileSelect(files[0]);
    }
  }, [handleFileSelect]);

  const handleInputChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(e.target.files || []);
    if (files.length > 0) {
      handleFileSelect(files[0]);
    }
  }, [handleFileSelect]);

  const openFileDialog = useCallback(() => {
    fileInputRef.current?.click();
  }, []);

  const formatFileSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  };

  return (
    <div className={clsx('relative', className)}>
      <div
        className={clsx(
          'border-2 border-dashed rounded-lg p-8 text-center transition-colors duration-200',
          isDragActive
            ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20'
            : error
            ? 'border-error-300 bg-error-50 dark:bg-error-900/20'
            : 'border-gray-300 dark:border-gray-600 bg-gray-50 dark:bg-gray-800/50 hover:bg-gray-100 dark:hover:bg-gray-800',
          isUploading && 'pointer-events-none opacity-75'
        )}
        onDragEnter={handleDragEnter}
        onDragLeave={handleDragLeave}
        onDragOver={handleDragOver}
        onDrop={handleDrop}
        onClick={openFileDialog}
        role="button"
        tabIndex={0}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            openFileDialog();
          }
        }}
      >
        <input
          ref={fileInputRef}
          type="file"
          accept={acceptedTypes.join(',')}
          onChange={handleInputChange}
          className="sr-only"
        />

        <div className="flex flex-col items-center space-y-4">
          {isUploading ? (
            <>
              <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
              <div className="w-full max-w-xs">
                <div className="bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                  <div
                    className="bg-primary-600 h-2 rounded-full transition-all duration-300"
                    style={{ width: `${uploadProgress}%` }}
                  ></div>
                </div>
                <p className="text-sm text-gray-600 dark:text-gray-400 mt-2">
                  Uploading... {uploadProgress}%
                </p>
              </div>
            </>
          ) : (
            <>
              <CloudArrowUpIcon className="h-12 w-12 text-gray-400" />
              <div>
                <p className="text-lg font-medium text-gray-900 dark:text-gray-100">
                  {isDragActive ? 'Drop your file here' : 'Upload a log file'}
                </p>
                <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                  Drag and drop or click to browse
                </p>
              </div>
            </>
          )}

          <div className="flex items-center space-x-4 text-xs text-gray-500 dark:text-gray-400">
            <span>Max size: {formatFileSize(maxSizeBytes)}</span>
            <span>•</span>
            <span>Supported: {acceptedTypes.join(', ')}</span>
          </div>

          {!isUploading && (
            <div className="flex items-center space-x-2 text-sm text-gray-600 dark:text-gray-400">
              <DocumentTextIcon className="h-4 w-4" />
              <span>Log files, text files, JSON files</span>
            </div>
          )}
        </div>
      </div>

      {error && (
        <div className="mt-3 bg-error-50 dark:bg-error-900/20 border border-error-200 dark:border-error-800 rounded-md p-3">
          <p className="text-sm text-error-700 dark:text-error-300">{error}</p>
        </div>
      )}
    </div>
  );
}