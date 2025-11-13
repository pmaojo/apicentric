'use client';

import { Button } from '@/components/ui/button';
import { SidebarTrigger } from '@/components/ui/sidebar';
import { WebSocketStatus } from '@/components/ui/websocket-status';
import { Play, Square } from 'lucide-react';
import * as React from 'react';

type HeaderProps = {
  title: string;
  isSimulatorRunning: boolean;
  onToggleSimulator: () => void;
};

export function Header({ title, isSimulatorRunning, onToggleSimulator }: HeaderProps) {
  return (
    <header className="sticky top-0 z-10 flex h-16 items-center gap-4 border-b bg-background/80 px-4 backdrop-blur-sm md:px-6">
      <div className="md:hidden">
        <SidebarTrigger />
      </div>
      <h1 className="text-xl font-semibold tracking-tight">{title}</h1>
      <div className="ml-auto flex items-center gap-3">
        <WebSocketStatus />
        {isSimulatorRunning ? (
          <Button variant="destructive" onClick={onToggleSimulator} data-testid="simulator-toggle">
            <Square className="mr-2 h-4 w-4" />
            Stop Simulator
          </Button>
        ) : (
          <Button className="bg-accent text-accent-foreground hover:bg-accent/90" onClick={onToggleSimulator} data-testid="simulator-toggle">
            <Play className="mr-2 h-4 w-4" />
            Start Simulator
          </Button>
        )}
      </div>
    </header>
  );
}
