// components/ServerDetails.tsx
import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

const LabelValuePair = ({ label, value }: { label: string; value: string }) => (
  <div className="flex justify-between">
    <span className="text-sm font-medium text-muted-foreground">{label}</span>
    <span className="text-sm">{value}</span>
  </div>
);

export const ServerDetails = () => (
  <Card>
    <CardHeader>
      <CardTitle>Server Details</CardTitle>
    </CardHeader>
    <CardContent className="space-y-2">
      <LabelValuePair label="Hostname" value="app-server-01" />
      <LabelValuePair label="Environment" value="Production" />
      <LabelValuePair label="Region" value="us-east-1" />
      <LabelValuePair label="Provider" value="AWS" />
      <LabelValuePair label="Instance Type" value="t3.xlarge" />
      <LabelValuePair label="Provisioned IOPS" value="5000" />
    </CardContent>
  </Card>
);
