/**
 * WebSocket Manager - Infrastructure Layer
 * 
 * Manages WebSocket connections and message routing.
 * This is the single point of WebSocket management.
 */

import { logger } from './logger';

export interface WebSocketMessage {
  type: string;
  data?: any;
  timestamp?: string;
}

export interface WebSocketManagerInterface {
  connect(): void;
  disconnect(): void;
  subscribe(messageType: string, callback: (data: any) => void): () => void;
  send(message: any): void;
  isConnected(): boolean;
}

export class WebSocketManager implements WebSocketManagerInterface {
  private ws: WebSocket | null = null;
  private subscribers = new Map<string, Set<(data: any) => void>>();
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;
  private url: string;

  constructor(url: string = 'ws://localhost:8080/ws') {
    this.url = url;
  }

  connect(): void {
    if (this.ws?.readyState === WebSocket.CONNECTING) {
      return;
    }

    try {
      this.ws = new WebSocket(this.url);
      
      this.ws.onopen = () => {
        logger.log('WebSocket connected');
        this.reconnectAttempts = 0;
      };

      this.ws.onmessage = (event) => {
        try {
          const message: WebSocketMessage = JSON.parse(event.data);
          this.handleMessage(message);
        } catch (error) {
          logger.error('Failed to parse WebSocket message:', error);
        }
      };

      this.ws.onclose = (event) => {
        logger.log('WebSocket closed:', event.code);
        this.ws = null;
        
        if (event.code !== 1000 && this.reconnectAttempts < this.maxReconnectAttempts) {
          setTimeout(() => {
            this.reconnectAttempts++;
            this.connect();
          }, this.reconnectDelay * Math.pow(2, this.reconnectAttempts));
        }
      };

      this.ws.onerror = (error) => {
        logger.error('WebSocket error:', error);
      };

    } catch (error) {
      logger.error('Failed to create WebSocket connection:', error);
    }
  }

  disconnect(): void {
    if (this.ws) {
      this.ws.close(1000, 'Manual disconnect');
      this.ws = null;
    }
    this.reconnectAttempts = 0;
  }

  subscribe(messageType: string, callback: (data: any) => void): () => void {
    if (!this.subscribers.has(messageType)) {
      this.subscribers.set(messageType, new Set());
    }
    
    this.subscribers.get(messageType)!.add(callback);

    // Return unsubscribe function
    return () => {
      const typeSubscribers = this.subscribers.get(messageType);
      if (typeSubscribers) {
        typeSubscribers.delete(callback);
        if (typeSubscribers.size === 0) {
          this.subscribers.delete(messageType);
        }
      }
    };
  }

  send(message: any): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    } else {
      logger.warn('WebSocket is not connected');
    }
  }

  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  private handleMessage(message: WebSocketMessage): void {
    const subscribers = this.subscribers.get(message.type);
    if (subscribers) {
      subscribers.forEach(callback => {
        try {
          callback(message.data);
        } catch (error) {
          logger.error(`Error in WebSocket subscriber for ${message.type}:`, error);
        }
      });
    }
  }
}

// Export singleton instance
export const webSocketManager = new WebSocketManager();