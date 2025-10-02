/**
 * Frontend logging utilities for LogLens
 * Provides structured logging with different levels and easy error spotting
 */

export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3,
}

interface LogEntry {
  timestamp: string;
  level: LogLevel;
  component: string;
  message: string;
  data?: any;
}

class Logger {
  private static instance: Logger;
  private logs: LogEntry[] = [];
  private maxLogs: number = 1000;
  private logLevel: LogLevel = LogLevel.INFO;

  private constructor() {
    // Initialize logger
    console.info('[LogLens] Logger initialized');
  }

  public static getInstance(): Logger {
    if (!Logger.instance) {
      Logger.instance = new Logger();
    }
    return Logger.instance;
  }

  public setLogLevel(level: LogLevel): void {
    this.logLevel = level;
    this.info('Logger', `Log level set to ${LogLevel[level]}`);
  }

  private shouldLog(level: LogLevel): boolean {
    return level >= this.logLevel;
  }

  private createLogEntry(level: LogLevel, component: string, message: string, data?: any): LogEntry {
    const entry: LogEntry = {
      timestamp: new Date().toISOString(),
      level,
      component,
      message,
      data,
    };

    // Add to logs array (keep only last maxLogs)
    this.logs.push(entry);
    if (this.logs.length > this.maxLogs) {
      this.logs = this.logs.slice(-this.maxLogs);
    }

    return entry;
  }

  private log(level: LogLevel, component: string, message: string, data?: any): void {
    if (!this.shouldLog(level)) {
      return;
    }

    const entry = this.createLogEntry(level, component, message, data);
    const levelName = LogLevel[level];
    const prefix = `[${entry.timestamp}] [${levelName}] [${component}]`;

    switch (level) {
      case LogLevel.DEBUG:
        console.debug(prefix, message, data || '');
        break;
      case LogLevel.INFO:
        console.info(prefix, message, data || '');
        break;
      case LogLevel.WARN:
        console.warn(prefix, message, data || '');
        break;
      case LogLevel.ERROR:
        console.error(prefix, message, data || '');
        // Stack trace for errors
        if (data && data instanceof Error) {
          console.error('Stack trace:', data.stack);
        }
        break;
    }
  }

  public debug(component: string, message: string, data?: any): void {
    this.log(LogLevel.DEBUG, component, message, data);
  }

  public info(component: string, message: string, data?: any): void {
    this.log(LogLevel.INFO, component, message, data);
  }

  public warn(component: string, message: string, data?: any): void {
    this.log(LogLevel.WARN, component, message, data);
  }

  public error(component: string, message: string, data?: any): void {
    this.log(LogLevel.ERROR, component, message, data);
  }

  public getLogs(): LogEntry[] {
    return [...this.logs];
  }

  public getLogsByLevel(level: LogLevel): LogEntry[] {
    return this.logs.filter(log => log.level === level);
  }

  public clearLogs(): void {
    this.logs = [];
    this.info('Logger', 'Logs cleared');
  }

  public exportLogs(): string {
    return JSON.stringify(this.logs, null, 2);
  }
}

// Export singleton instance
export const logger = Logger.getInstance();

// Development helpers
export const logApiCall = (endpoint: string, method: string, status?: number, error?: any) => {
  if (error) {
    logger.error('API', `${method} ${endpoint} failed`, { status, error });
  } else {
    logger.info('API', `${method} ${endpoint}`, { status });
  }
};

export const logWebSocketEvent = (event: string, data?: any) => {
  logger.info('WebSocket', event, data);
};

export const logUserAction = (action: string, data?: any) => {
  logger.info('UserAction', action, data);
};

export const logPerformance = (operation: string, duration: number, data?: any) => {
  logger.info('Performance', `${operation} took ${duration}ms`, data);
};

// Error boundary logging
export const logErrorBoundary = (error: Error, errorInfo: React.ErrorInfo) => {
  logger.error('ErrorBoundary', 'React error caught', {
    error: error.message,
    stack: error.stack,
    componentStack: errorInfo.componentStack,
  });
};

export default logger;