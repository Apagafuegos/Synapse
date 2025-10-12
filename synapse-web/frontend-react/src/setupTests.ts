import '@testing-library/jest-dom';
import { jest } from '@jest/globals';

// Mock WebSocket
global.WebSocket = class MockWebSocket {
  constructor(url: string) {
    console.log('Mock WebSocket created for:', url);
  }

  close() {}
  send() {}

  static CONNECTING = 0;
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;

  readyState = MockWebSocket.OPEN;

  addEventListener() {}
  removeEventListener() {}
} as any;

// Mock IntersectionObserver
global.IntersectionObserver = class MockIntersectionObserver {
  constructor(_callback?: IntersectionObserverCallback, _options?: IntersectionObserverInit) {}

  observe() {}
  unobserve() {}
  disconnect() {}
} as any;

// Mock ResizeObserver
global.ResizeObserver = class MockResizeObserver {
  constructor(_callback?: ResizeObserverCallback) {}

  observe() {}
  unobserve() {}
  disconnect() {}
} as any;

// Mock matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: jest.fn().mockImplementation((query: any) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: jest.fn(), // deprecated
    removeListener: jest.fn(), // deprecated
    addEventListener: jest.fn(),
    removeEventListener: jest.fn(),
    dispatchEvent: jest.fn(),
  })),
});

// Mock localStorage
const localStorageMock = {
  getItem: jest.fn(),
  setItem: jest.fn(),
  removeItem: jest.fn(),
  clear: jest.fn(),
};
global.localStorage = localStorageMock as any;

// Suppress console warnings in tests
const originalError = console.error;
const beforeAll = (global as any).beforeAll || ((fn: () => void) => fn());
const afterAll = (global as any).afterAll || ((fn: () => void) => fn());

beforeAll(() => {
  (console as any).error = (...args: any[]) => {
    if (
      typeof args[0] === 'string' &&
      args[0].includes('Warning: ReactDOM.render is no longer supported')
    ) {
      return;
    }
    originalError.call(console, ...args);
  };
});

afterAll(() => {
  (console as any).error = originalError;
});