// components/MonitoringCard.tsx
import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Progress } from "@/components/ui/progress";

const ProgressBar = ({ label, value }: { label: string; value: number }) => (
  <div className="grid gap-1">
    <Label>{label}</Label>
    <Progress value={value} />
  </div>
);

export const MonitoringCard = () => (
  <Card>
    <CardHeader>
      <CardTitle>Monitoring</CardTitle>
    </CardHeader>
    <CardContent className="space-y-4">
      <ProgressBar label="CPU Utilization" value={75} />
      <ProgressBar label="Memory Usage" value={60} />
      <ProgressBar label="Disk Space" value={85} />
      <ProgressBar label="Network Traffic" value={45} />
    </CardContent>
  </Card>
);
