/**
 * Logger - Infrastructure Layer
 *
 * Simple wrapper around console to allow disabling logs in production.
 */

const isProduction = process.env.NODE_ENV === 'production';

export const logger = {
  log: (...args: any[]) => {
    if (!isProduction) {
      console.log(...args);
    }
  },
  info: (...args: any[]) => {
    if (!isProduction) {
      console.info(...args);
    }
  },
  warn: (...args: any[]) => {
    // Warnings are often important enough to keep in production,
    // but can be silenced if needed. For now we keep them.
    console.warn(...args);
  },
  error: (...args: any[]) => {
    // Errors should always be logged
    console.error(...args);
  },
  debug: (...args: any[]) => {
    if (!isProduction) {
      console.debug(...args);
    }
  }
};
