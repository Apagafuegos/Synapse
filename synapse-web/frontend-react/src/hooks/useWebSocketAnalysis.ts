import { useState, useEffect, useCallback, useRef } from 'react';

export interface AnalysisProgress {
  stage: string;
  progress: number; // 0.0 to 1.0
  message: string;
  elapsed_ms: number;
}

export interface AnalysisStats {
  total_lines: number;
  parsed_entries: number;
  filtered_entries: number;
  slimmed_entries: number;
  processing_time_ms: number;
  ai_analysis_time_ms: number;
}

export interface AnalysisComplete {
  analysis: any; // Full AnalysisResponse
  analysis_id: string;
  elapsed_ms: number;
  stats: AnalysisStats;
}

export interface WebSocketMessage {
  type: 'Progress' | 'Error' | 'Complete' | 'Cancelled' | 'Heartbeat';
  data: {
    stage?: string;
    progress?: number;
    message?: string;
    elapsed_ms?: number;
    error?: string;
    analysis?: any;
    analysis_id?: string;
    stats?: AnalysisStats;
    reason?: string;
    timestamp?: number;
  };
}

export interface UseWebSocketAnalysisOptions {
  projectId: string;
  fileId: string;
  provider: string;
  level: string;
  apiKey?: string;
  userContext?: string;
  onProgress?: (progress: AnalysisProgress) => void;
  onComplete?: (result: AnalysisComplete) => void;
  onError?: (error: string) => void;
  onCancel?: (reason: string) => void;
}

export function useWebSocketAnalysis(options: UseWebSocketAnalysisOptions) {
  const [isConnected, setIsConnected] = useState(false);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [progress, setProgress] = useState<AnalysisProgress | null>(null);
  const [result, setResult] = useState<AnalysisComplete | null>(null);
  const [error, setError] = useState<string | null>(null);

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  const connect = useCallback(() => {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = window.location.host;
    const params = new URLSearchParams({
      level: options.level,
      provider: options.provider,
      ...(options.apiKey && { api_key: options.apiKey }),
      ...(options.userContext && { user_context: options.userContext }),
    });

    const url = `${protocol}//${host}/api/projects/${options.projectId}/files/${options.fileId}/analyze/ws?${params}`;

    console.log('[WebSocket Analysis] Connecting to:', url);
    const ws = new WebSocket(url);
    wsRef.current = ws;

    ws.onopen = () => {
      console.log('[WebSocket Analysis] Connected');
      setIsConnected(true);
      setIsAnalyzing(true);
      setError(null);
    };

    ws.onmessage = (event) => {
      try {
        const message: WebSocketMessage = JSON.parse(event.data);
        console.log('[WebSocket Analysis] Message:', message);

        switch (message.type) {
          case 'Progress':
            const progressData: AnalysisProgress = {
              stage: message.data.stage || '',
              progress: message.data.progress || 0,
              message: message.data.message || '',
              elapsed_ms: message.data.elapsed_ms || 0,
            };
            setProgress(progressData);
            options.onProgress?.(progressData);
            break;

          case 'Complete':
            const completeData: AnalysisComplete = {
              analysis: message.data.analysis,
              analysis_id: message.data.analysis_id || '',
              elapsed_ms: message.data.elapsed_ms || 0,
              stats: message.data.stats || {} as AnalysisStats,
            };
            setResult(completeData);
            setIsAnalyzing(false);
            options.onComplete?.(completeData);
            break;

          case 'Error':
            const errorMsg = message.data.error || 'Unknown error occurred';
            setError(errorMsg);
            setIsAnalyzing(false);
            options.onError?.(errorMsg);
            break;

          case 'Cancelled':
            const reason = message.data.reason || 'Analysis cancelled';
            setIsAnalyzing(false);
            options.onCancel?.(reason);
            break;

          case 'Heartbeat':
            // Just acknowledge heartbeat
            console.log('[WebSocket Analysis] Heartbeat received');
            break;
        }
      } catch (err) {
        console.error('[WebSocket Analysis] Failed to parse message:', err);
      }
    };

    ws.onerror = (event) => {
      console.error('[WebSocket Analysis] Error:', event);
      setError('WebSocket connection error');
      setIsAnalyzing(false);
    };

    ws.onclose = (event) => {
      console.log('[WebSocket Analysis] Closed:', event.code, event.reason);
      setIsConnected(false);
      setIsAnalyzing(false);

      // Auto-reconnect if unexpected close and no error
      if (event.code !== 1000 && !error) {
        reconnectTimeoutRef.current = setTimeout(() => {
          console.log('[WebSocket Analysis] Attempting to reconnect...');
          connect();
        }, 5000);
      }
    };
  }, [options, error]);

  const cancel = useCallback(() => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      console.log('[WebSocket Analysis] Sending cancel message');
      wsRef.current.send('cancel');
      setIsAnalyzing(false);
    }
  }, []);

  const disconnect = useCallback(() => {
    if (wsRef.current) {
      wsRef.current.close(1000, 'User initiated disconnect');
      wsRef.current = null;
    }
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }
  }, []);

  useEffect(() => {
    return () => {
      disconnect();
    };
  }, [disconnect]);

  return {
    isConnected,
    isAnalyzing,
    progress,
    result,
    error,
    connect,
    cancel,
    disconnect,
  };
}
