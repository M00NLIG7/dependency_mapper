// components/ServerOverview.tsx
import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";

const LabelValuePair = ({ label, value }: { label: string; value: string }) => (
  <div className="flex justify-between">
    <span className="text-sm font-medium text-muted-foreground">{label}</span>
    <span className="text-sm">{value}</span>
  </div>
);

const SpecsGrid = () => (
  <div className="grid grid-cols-2 gap-4">
    <LabelValuePair label="CPU" value="Intel Core i7-10700K" />
    <LabelValuePair label="RAM" value="32GB DDR4" />
    <LabelValuePair label="Storage" value="1TB SSD" />
    <LabelValuePair label="OS" value="Ubuntu 22.04 LTS" />
  </div>
);

export const ServerOverview = () => (
  <Card>
    <CardHeader>
      <CardTitle>Overview</CardTitle>
    </CardHeader>
    <CardContent className="space-y-4">
      <SpecsGrid />
      <Separator />
      <LabelValuePair label="Uptime" value="June 1, 2023 - 12:00 PM" />
      <LabelValuePair label="IP Address" value="192.168.1.100" />
      <LabelValuePair label="Location" value="New York, USA" />
    </CardContent>
  </Card>
);
