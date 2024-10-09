// app/dashboard/page.tsx
import React from 'react';
import { Server, MoreVertical } from 'lucide-react';
import { Button } from "@/components/ui/button";
import { ServerOverview } from '@/components/ServerOverview';
import { DependenciesTable } from '@/components/DependenciesTable';
import { ServerDetails } from '@/components/ServerDetails';
import { MonitoringCard } from '@/components/MonitoringCard';

const NavbarBrand = () => (
  <a className="flex items-center gap-2 font-semibold" href="#">
    <Server className="h-6 w-6" />
    <span>Acme Server</span>
  </a>
);

const Navbar = () => (
  <header className="sticky top-0 z-30 flex h-14 items-center gap-4 border-b bg-background px-4 sm:h-16 sm:px-6">
    <NavbarBrand />
    <div className="ml-auto flex items-center gap-2">
      <Button variant="ghost" size="icon">
        <MoreVertical className="h-4 w-4" />
      </Button>
    </div>
  </header>
);

export default function NodeDashboard() {
  return (
    <div className="flex min-h-screen flex-col bg-muted/40">
      <Navbar />
      <main className="flex-1 p-4 sm:p-6">
        <div className="grid gap-6 lg:grid-cols-[1fr_300px]">
          <div>
            <div className="grid gap-6">
              <ServerOverview />
              <DependenciesTable />
            </div>
          </div>
          <div className="grid gap-6">
            <ServerDetails />
            <MonitoringCard />
          </div>
        </div>
      </main>
    </div>
  );
}
