
'use client';

import {
  SidebarMenu,
  SidebarMenuItem,
  SidebarMenuButton,
} from '@/components/ui/sidebar';
import type { View } from '@/lib/types';
import {
  LayoutDashboard,
  Server,
  FileText,
  Sparkles,
  Code,
  FileCheck,
  RadioTower,
  Pencil,
  Wrench,
  Asterisk,
  Settings,
  ShoppingBag,
  Cpu,
} from 'lucide-react';
import { useSidebar } from '@/components/ui/sidebar';

/**
 * @fileoverview Provides the primary navigation menu for the sidebar.
 */

/**
 * Props for the SidebarNav component.
 * @typedef {object} SidebarNavProps
 * @property {View} activeView - The currently active view.
 * @property {(view: View) => void} setActiveView - Function to set the active view.
 */
type SidebarNavProps = {
  activeView: View;
  setActiveView: (view: View) => void;
};

/**
 * Navigation items to be displayed in the sidebar.
 * Each item has an id, a label for display, and an icon component.
 * @const
 */
const navItems = [
    { id: 'dashboard', label: 'Dashboard', icon: LayoutDashboard },
    { id: 'services', label: 'Services', icon: Server },
    { id: 'iot', label: 'IoT Twins', icon: Cpu },
    { id: 'marketplace', label: 'Marketplace', icon: ShoppingBag },
    { id: 'recording', label: 'Recording', icon: RadioTower },
    { id: 'ai-generator', label: 'AI Generator', icon: Sparkles },
    { id: 'plugin-generator', label: 'Plugin Generator', icon: Wrench },
    { id: 'contract-testing', label: 'Contract Testing', icon: FileCheck },
    { id: 'code-generator', label: 'Code Generator', icon: Code },
    { id: 'logs', label: 'Logs', icon: FileText },
    { id: 'configuration', label: 'Configuration', icon: Settings },
] as const;


/**
 * Renders the main sidebar navigation menu.
 * @param {SidebarNavProps} props - The component props.
 * @returns {React.ReactElement} The rendered sidebar navigation.
 */
export function SidebarNav({ activeView, setActiveView }: SidebarNavProps) {
  const { setOpenMobile } = useSidebar();
  
  /**
   * Handles navigation to a new view and closes the mobile sidebar.
   * @param {View} view - The view to navigate to.
   */
  const handleNavigation = (view: View) => {
    setActiveView(view);
    setOpenMobile(false);
  }

  return (
    <SidebarMenu>
      {navItems.map((item) => (
        <SidebarMenuItem key={item.id}>
          <SidebarMenuButton
            onClick={() => handleNavigation(item.id)}
            isActive={activeView === item.id}
            tooltip={{children: item.label}}
            data-testid={`sidebar-${item.id}`}
          >
            <item.icon />
            <span>{item.label}</span>
          </SidebarMenuButton>
        </SidebarMenuItem>
      ))}
    </SidebarMenu>
  );
}
