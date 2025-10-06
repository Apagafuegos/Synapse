# WebSocket Analysis Integration Guide

This guide shows how to integrate the WebSocket-based real-time analysis feature into your pages.

## Components Created

1. **`useWebSocketAnalysis` Hook** (`src/hooks/useWebSocketAnalysis.ts`)
   - Manages WebSocket connection to analysis endpoint
   - Handles progress updates, completion, errors, and cancellation
   - Provides connection state and control methods

2. **`AnalysisProgress` Component** (`src/components/AnalysisProgress.tsx`)
   - Displays real-time progress with visual progress bar
   - Shows analysis statistics (lines parsed, filtered, etc.)
   - Includes cancel button for stopping analysis

3. **`WebSocketAnalysisPanel` Component** (`src/components/WebSocketAnalysisPanel.tsx`)
   - Complete integration example
   - Combines hook + progress component + UI controls
   - Ready to drop into any page

## Integration Example

### Option 1: Use the Pre-built Panel

```tsx
import { WebSocketAnalysisPanel } from '@/components/WebSocketAnalysisPanel';

function ProjectDetail() {
  const { id: projectId } = useParams();

  return (
    <div>
      {/* ... existing code ... */}

      {/* Add WebSocket Analysis Panel */}
      <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6">
        <WebSocketAnalysisPanel
          projectId={projectId}
          fileId={selectedFile.id}
          fileName={selectedFile.filename}
          onComplete={() => {
            // Refresh analyses list
            queryClient.invalidateQueries(['project-analyses', projectId]);
          }}
        />
      </div>
    </div>
  );
}
```

### Option 2: Custom Integration with Hook

```tsx
import { useState } from 'react';
import { useWebSocketAnalysis } from '@/hooks/useWebSocketAnalysis';
import { AnalysisProgress } from '@/components/AnalysisProgress';

function CustomAnalysisView() {
  const [showProgress, setShowProgress] = useState(false);

  const {
    isConnected,
    isAnalyzing,
    progress,
    result,
    error,
    connect,
    cancel,
  } = useWebSocketAnalysis({
    projectId: 'your-project-id',
    fileId: 'your-file-id',
    provider: 'openrouter',
    level: 'ERROR',
    userContext: 'Optional context about the logs',
    onProgress: (prog) => {
      console.log(`${prog.stage}: ${prog.message}`);
    },
    onComplete: (res) => {
      console.log('Analysis complete!', res.analysis_id);
      // Navigate to results or update UI
    },
    onError: (err) => {
      console.error('Analysis failed:', err);
    },
    onCancel: (reason) => {
      console.log('Cancelled:', reason);
    },
  });

  return (
    <div>
      {!isAnalyzing && (
        <button onClick={() => {
          setShowProgress(true);
          connect();
        }}>
          Start Analysis
        </button>
      )}

      {isAnalyzing && progress && (
        <AnalysisProgress
          progress={progress}
          stats={result?.stats}
          onCancel={cancel}
        />
      )}

      {result && (
        <div>
          <h3>Analysis Complete!</h3>
          <p>ID: {result.analysis_id}</p>
          <p>Time: {(result.elapsed_ms / 1000).toFixed(2)}s</p>
        </div>
      )}

      {error && (
        <div className="error">
          Error: {error}
        </div>
      )}
    </div>
  );
}
```

## Integration into ProjectDetail.tsx

Add a toggle to choose between traditional and WebSocket analysis:

```tsx
function ProjectDetail() {
  // ... existing state ...
  const [useWebSocketAnalysis, setUseWebSocketAnalysis] = useState(true);
  const [wsAnalysisFile, setWsAnalysisFile] = useState<string | null>(null);

  // In the Analysis Options Panel, add:
  <div className="mt-4 p-4 bg-gray-50 dark:bg-gray-700 rounded-lg space-y-4">
    <h4 className="text-sm font-medium text-gray-900 dark:text-white">
      Analysis Options
    </h4>

    {/* WebSocket Toggle */}
    <div className="flex items-center">
      <input
        id="use-websocket"
        type="checkbox"
        checked={useWebSocketAnalysis}
        onChange={(e) => setUseWebSocketAnalysis(e.target.checked)}
        className="h-4 w-4 text-primary-600 focus:ring-primary-500 border-gray-300 rounded"
      />
      <label htmlFor="use-websocket" className="ml-2 block text-sm text-gray-700 dark:text-gray-300">
        Use real-time progress (WebSocket)
      </label>
    </div>

    {/* ... existing options ... */}
  </div>

  // Modify the Analyze button:
  <button
    onClick={() => {
      if (useWebSocketAnalysis) {
        setWsAnalysisFile(file.id);
      } else {
        handleStartAnalysis(file.id, { userContext, timeoutSeconds });
      }
    }}
    className="btn-primary"
  >
    Analyze
  </button>

  // Add WebSocket Analysis Panel when active:
  {wsAnalysisFile && useWebSocketAnalysis && (
    <div className="mt-4">
      <WebSocketAnalysisPanel
        projectId={id!}
        fileId={wsAnalysisFile}
        fileName={files?.find(f => f.id === wsAnalysisFile)?.filename || ''}
        onComplete={() => {
          setWsAnalysisFile(null);
          queryClient.invalidateQueries(['project-analyses', id]);
        }}
      />
    </div>
  )}
}
```

## Message Types

The WebSocket endpoint sends these message types:

```typescript
interface WebSocketMessage {
  type: 'Progress' | 'Error' | 'Complete' | 'Cancelled' | 'Heartbeat';
  data: {
    // Progress
    stage?: string;           // 'reading_file', 'parsing', 'ai_analysis', etc.
    progress?: number;        // 0.0 to 1.0
    message?: string;         // Human-readable status message
    elapsed_ms?: number;      // Milliseconds since start

    // Complete
    analysis?: any;           // Full analysis result
    analysis_id?: string;     // Database ID of completed analysis
    stats?: AnalysisStats;    // Processing statistics

    // Error
    error?: string;           // Error message

    // Cancelled
    reason?: string;          // Cancellation reason

    // Heartbeat
    timestamp?: number;       // Server timestamp
  };
}
```

## Sending Cancel Message

To cancel an ongoing analysis, send a text message "cancel":

```typescript
if (ws.readyState === WebSocket.OPEN) {
  ws.send('cancel');
}
```

The hook's `cancel()` function handles this automatically.

## Backend Endpoint

The WebSocket connects to:

```
ws://localhost:8080/api/projects/{project_id}/files/{file_id}/analyze/ws?provider={provider}&level={level}
```

Query parameters:
- `provider`: AI provider (e.g., "openrouter", "claude", "openai")
- `level`: Log level filter (e.g., "ERROR", "WARN", "INFO")
- `api_key`: (optional) Override API key
- `user_context`: (optional) Additional context for analysis

## Styling

The components use Tailwind CSS classes with dark mode support. Key colors:
- Primary: `primary-500`, `primary-600` (customizable in tailwind.config.js)
- Success: `green-*`
- Error: `red-*`
- Progress bar: Gradient from `primary-500` to `primary-600`

## Testing

1. **Manual Testing**:
   - Start the backend: `cargo run`
   - Start the frontend: `npm run dev`
   - Upload a log file
   - Click "Analyze" with WebSocket option enabled
   - Observe real-time progress updates

2. **Error Testing**:
   - Test with invalid API key
   - Test connection drop during analysis
   - Test cancel functionality

3. **Edge Cases**:
   - Very large files (watch progress granularity)
   - Very small files (ensure quick stages don't skip)
   - Network interruptions (auto-reconnect)

## Comparison: Traditional vs WebSocket Analysis

| Feature | Traditional (REST) | WebSocket |
|---------|-------------------|-----------|
| Progress Updates | Polling required | Real-time push |
| User Experience | Spinner only | Detailed progress |
| Cancellation | Not supported | Instant |
| Network Efficiency | Multiple requests | Single connection |
| Complexity | Simple | Moderate |

## Future Enhancements

- [ ] Add progress percentage to browser title
- [ ] Desktop notifications on completion
- [ ] Persist progress across page refreshes
- [ ] Multiple concurrent analyses tracking
- [ ] Audio notification on completion
- [ ] Estimated time remaining calculation
