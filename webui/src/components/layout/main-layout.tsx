import {
  SidebarProvider,
  Sidebar,
  SidebarHeader,
  SidebarContent,
  SidebarFooter,
  SidebarInset,
} from '@/components/ui/sidebar';
import type { View } from '@/lib/types';
import { ApicentricIcon } from '../icons';
import { SidebarNav } from './sidebar-nav';
import { Header } from './header';

type MainLayoutProps = {
  children: React.ReactNode;
  activeView: View;
  setActiveView: (view: View) => void;
  title: string;
  isSimulatorRunning: boolean;
  onToggleAllServices: () => void;
};

export function MainLayout({
  children,
  activeView,
  setActiveView,
  title,
  isSimulatorRunning,
  onToggleAllServices,
}: MainLayoutProps) {
  return (
    <SidebarProvider>
      <Sidebar>
        <SidebarHeader>
          <div className="flex items-center gap-2">
            <ApicentricIcon className="h-6 w-6 text-primary" />
            <span className="text-lg font-semibold">Apicentric</span>
          </div>
        </SidebarHeader>
        <SidebarContent>
          <SidebarNav activeView={activeView} setActiveView={setActiveView} />
        </SidebarContent>
        <SidebarFooter>
            {/* Can add footer items here if needed */}
        </SidebarFooter>
      </Sidebar>
      <SidebarInset className="flex flex-col">
        <Header 
            title={title} 
            isSimulatorRunning={isSimulatorRunning}
            onToggleSimulator={onToggleAllServices}
        />
        <main className="flex-1 overflow-y-auto p-4 md:p-6">
            {children}
        </main>
      </SidebarInset>
    </SidebarProvider>
  );
}
