'use client';

import { SidebarTrigger } from '@/components/ui/sidebar';
import { WebSocketStatus } from '@/components/ui/websocket-status';
import * as React from 'react';


type HeaderProps = {
  title: string;
};

export function Header({ title }: HeaderProps) {
  return (
    <header className="sticky top-0 z-10 flex h-16 items-center gap-4 border-b bg-background/80 px-4 backdrop-blur-sm md:px-6">
      <div className="md:hidden">
        <SidebarTrigger />
      </div>
      <h1 className="text-xl font-semibold tracking-tight">{title}</h1>
      <div className="ml-auto flex items-center gap-3">
        <WebSocketStatus />
      </div>
    </header>
  );
}
