/**
 * @fileoverview Performance monitoring utilities for tracking and optimizing application performance.
 */

/**
 * Measures the performance of a function execution
 * @param name - The name of the operation being measured
 * @param fn - The function to measure
 * @returns The result of the function execution
 */
export async function measurePerformance<T>(
  name: string,
  fn: () => Promise<T> | T
): Promise<T> {
  const start = performance.now();
  try {
    const result = await fn();
    const duration = performance.now() - start;
    
    if (process.env.NODE_ENV === 'development') {
      console.log(`[Performance] ${name}: ${duration.toFixed(2)}ms`);
    }
    
    // Report to analytics in production
    if (typeof window !== 'undefined' && 'performance' in window) {
      performance.mark(`${name}-end`);
      performance.measure(name, `${name}-start`, `${name}-end`);
    }
    
    return result;
  } catch (error) {
    const duration = performance.now() - start;
    console.error(`[Performance] ${name} failed after ${duration.toFixed(2)}ms`, error);
    throw error;
  }
}

/**
 * Reports Web Vitals metrics
 */
export function reportWebVitals(metric: any) {
  if (process.env.NODE_ENV === 'development') {
    console.log('[Web Vitals]', metric);
  }
  
  // Send to analytics endpoint in production
  if (process.env.NODE_ENV === 'production') {
    // Example: send to analytics service
    // fetch('/api/analytics', {
    //   method: 'POST',
    //   body: JSON.stringify(metric),
    // });
  }
}

/**
 * Debounce function to limit the rate of function execution
 * @param fn - The function to debounce
 * @param delay - The delay in milliseconds
 * @returns The debounced function
 */
export function debounce<T extends (...args: any[]) => any>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: NodeJS.Timeout;
  
  return function (this: any, ...args: Parameters<T>) {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn.apply(this, args), delay);
  };
}

/**
 * Throttle function to limit the rate of function execution
 * @param fn - The function to throttle
 * @param limit - The time limit in milliseconds
 * @returns The throttled function
 */
export function throttle<T extends (...args: any[]) => any>(
  fn: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle: boolean;
  
  return function (this: any, ...args: Parameters<T>) {
    if (!inThrottle) {
      fn.apply(this, args);
      inThrottle = true;
      setTimeout(() => (inThrottle = false), limit);
    }
  };
}

/**
 * Lazy load a component with retry logic
 * @param importFn - The dynamic import function
 * @param retries - Number of retries on failure
 * @returns Promise that resolves to the component
 */
export function lazyWithRetry<T extends React.ComponentType<any>>(
  importFn: () => Promise<{ default: T }>,
  retries = 3
): Promise<{ default: T }> {
  return new Promise((resolve, reject) => {
    importFn()
      .then(resolve)
      .catch((error) => {
        if (retries === 0) {
          reject(error);
          return;
        }
        
        setTimeout(() => {
          lazyWithRetry(importFn, retries - 1).then(resolve, reject);
        }, 1000);
      });
  });
}
