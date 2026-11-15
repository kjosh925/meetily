'use client';

import React from 'react';
import { useSidebar } from '@/components/Sidebar/SidebarProvider';
import { MentionMonitorBar } from '@/components/MentionMonitorBar';
import { MentionCatchUp } from '@/components/MentionCatchUp';

interface MainContentProps {
  children: React.ReactNode;
}

const MainContent: React.FC<MainContentProps> = ({ children }) => {
  const { isCollapsed } = useSidebar();

  return (
    <main
      className={`flex-1 min-w-0 overflow-hidden transition-all duration-300 ${
        isCollapsed ? 'ml-16' : 'ml-64'
      }`}
    >
      <div className="pl-8 min-w-0 w-full max-w-full overflow-hidden">
        <MentionMonitorBar />
        <MentionCatchUp />
        {children}
      </div>
    </main>
  );
};

export default MainContent;
