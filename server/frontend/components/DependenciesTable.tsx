'use client'

// components/DependenciesTable.tsx
import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";

interface Dependency {
  module: string;
  remoteIp: string;
  remotePort: string;
  status: string;
}

const DependencyRow = ({ dep }: { dep: Dependency }) => (
  <TableRow>
    <TableCell>
      <div className="font-medium">{dep.module}</div>
      <div className="text-sm text-muted-foreground">{dep.remoteIp}</div>
    </TableCell>
    <TableCell>
      <div className="font-medium">UDP</div>
      <div className="text-sm text-muted-foreground">{dep.remotePort}</div>
    </TableCell>
    <TableCell>
      <Badge variant={dep.status === 'Connected' ? 'secondary' : 'outline'}>{dep.status}</Badge>
    </TableCell>
  </TableRow>
);

export const DependenciesTable = () => {
  const [deps, setDeps] = React.useState<Dependency[]>([]);

  React.useEffect(() => {
    const fetchDeps = async () => {
      const res = await fetch('/update-dependencies');
      const data = await res.json();
      setDeps(data);
    };

    fetchDeps();
    const interval = setInterval(fetchDeps, 30000);
    return () => clearInterval(interval);
  }, []);

  return (
    <Card>
      <CardHeader>
        <CardTitle>Dependencies</CardTitle>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Connection</TableHead>
              <TableHead>Status</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {deps.map((dep, index) => (
              <DependencyRow key={index} dep={dep} />
            ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  );
};
