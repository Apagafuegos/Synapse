import { createContext, useContext, useEffect, useRef, useState, ReactNode, useCallback } from 'react';
import type { WebSocketMessage, AnalysisStatusUpdate, SystemStatusUpdate } from '@/types';

type WebSocketStatus = 'connecting' | 'connected' | 'disconnected' | 'error';

interface WebSocketContextValue {
  status: WebSocketStatus;
  lastMessage: WebSocketMessage | null;
  sendMessage: (message: WebSocketMessage) => void;
  subscribe: (analysisId: string) => void;
  unsubscribe: (analysisId: string) => void;
}

const WebSocketContext = createContext<WebSocketContextValue | undefined>(undefined);

interface WebSocketProviderProps {
  children: ReactNode;
  url?: string;
}

export function WebSocketProvider({
  children,
  url = `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}/ws`
}: WebSocketProviderProps) {
  const [status, setStatus] = useState<WebSocketStatus>('disconnected');
  const [lastMessage, setLastMessage] = useState<WebSocketMessage | null>(null);

  const websocketRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const maxReconnectAttempts = 5;

  const connect = useCallback(() => {
    if (websocketRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    setStatus('connecting');
    console.log('[WebSocket] Connecting to:', url);

    try {
      const ws = new WebSocket(url);
      websocketRef.current = ws;

      ws.onopen = () => {
        console.log('[WebSocket] Connected');
        setStatus('connected');
        reconnectAttemptsRef.current = 0;
      };

      ws.onmessage = (event) => {
        try {
          const message: WebSocketMessage = JSON.parse(event.data);
          console.log('[WebSocket] Message received:', message);
          setLastMessage(message);
        } catch (error) {
          console.error('[WebSocket] Failed to parse message:', error);
        }
      };

      ws.onerror = (error) => {
        console.error('[WebSocket] Error:', error);
        setStatus('error');
      };

      ws.onclose = (event) => {
        console.log('[WebSocket] Connection closed:', event.code, event.reason);
        setStatus('disconnected');
        websocketRef.current = null;

        // Attempt to reconnect if not intentionally closed
        if (!event.wasClean && reconnectAttemptsRef.current < maxReconnectAttempts) {
          const delay = Math.pow(2, reconnectAttemptsRef.current) * 1000; // Exponential backoff
          console.log(`[WebSocket] Reconnecting in ${delay}ms...`);

          reconnectTimeoutRef.current = setTimeout(() => {
            reconnectAttemptsRef.current++;
            connect();
          }, delay);
        }
      };
    } catch (error) {
      console.error('[WebSocket] Failed to create connection:', error);
      setStatus('error');
    }
  }, [url]);

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    if (websocketRef.current) {
      websocketRef.current.close(1000, 'Client disconnect');
      websocketRef.current = null;
    }

    setStatus('disconnected');
  }, []);

  const sendMessage = useCallback((message: WebSocketMessage) => {
    if (websocketRef.current?.readyState === WebSocket.OPEN) {
      try {
        websocketRef.current.send(JSON.stringify(message));
        console.log('[WebSocket] Message sent:', message);
      } catch (error) {
        console.error('[WebSocket] Failed to send message:', error);
      }
    } else {
      console.warn('[WebSocket] Cannot send message: connection not open');
    }
  }, []);

  const subscribe = useCallback((analysisId: string) => {
    sendMessage({
      type: 'subscribe',
      analysis_id: analysisId,
    } as WebSocketMessage & { analysis_id: string });
  }, [sendMessage]);

  const unsubscribe = useCallback((analysisId: string) => {
    sendMessage({
      type: 'unsubscribe',
      analysis_id: analysisId,
    } as WebSocketMessage & { analysis_id: string });
  }, [sendMessage]);

  useEffect(() => {
    connect();

    return () => {
      disconnect();
    };
  }, [connect, disconnect]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
    };
  }, []);

  const value: WebSocketContextValue = {
    status,
    lastMessage,
    sendMessage,
    subscribe,
    unsubscribe,
  };

  return (
    <WebSocketContext.Provider value={value}>
      {children}
    </WebSocketContext.Provider>
  );
}

export function useWebSocket() {
  const context = useContext(WebSocketContext);
  if (context === undefined) {
    throw new Error('useWebSocket must be used within a WebSocketProvider');
  }
  return context;
}

// Specialized hook for analysis status updates
export function useAnalysisStatus(analysisId?: string) {
  const { lastMessage, subscribe, unsubscribe } = useWebSocket();
  const [analysisStatus, setAnalysisStatus] = useState<AnalysisStatusUpdate | null>(null);

  useEffect(() => {
    if (analysisId) {
      subscribe(analysisId);
      return () => unsubscribe(analysisId);
    }
  }, [analysisId, subscribe, unsubscribe]);

  useEffect(() => {
    if (lastMessage?.type === 'analysis_status') {
      const statusUpdate = lastMessage as AnalysisStatusUpdate;
      if (!analysisId || statusUpdate.analysis_id === analysisId) {
        setAnalysisStatus(statusUpdate);
      }
    }
  }, [lastMessage, analysisId]);

  return analysisStatus;
}

// Hook for system status updates
export function useSystemStatus() {
  const { lastMessage } = useWebSocket();
  const [systemStatus, setSystemStatus] = useState<SystemStatusUpdate | null>(null);

  useEffect(() => {
    if (lastMessage?.type === 'system_status') {
      setSystemStatus(lastMessage as SystemStatusUpdate);
    }
  }, [lastMessage]);

  return systemStatus;
}