// src/components/NetworkGraph/Sidebar.tsx
import React from 'react';
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { X } from 'lucide-react';

interface SidebarProps {
    isSidebarOpen: boolean;
    toggleSidebar: () => void;
    selectedTypes: string[];
    setSelectedTypes: React.Dispatch<React.SetStateAction<string[]>>;
    regexFilter: string;
    setRegexFilter: React.Dispatch<React.SetStateAction<string>>;
    applyFilters: () => void;
}

const Sidebar: React.FC<SidebarProps> = ({
    isSidebarOpen,
    toggleSidebar,
    selectedTypes,
    setSelectedTypes,
    regexFilter,
    setRegexFilter,
    applyFilters
}) => {
    return (
        <Card className={`fixed top-0 left-0 h-full w-64 shadow-lg transition-transform duration-300 ease-in-out z-50 ${isSidebarOpen ? 'translate-x-0' : '-translate-x-full'}`}>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">Filters</CardTitle>
                <Button
                    variant="ghost"
                    size="icon"
                    onClick={toggleSidebar}
                    className="h-8 w-8 p-0"
                >
                    <X className="h-4 w-4" />
                </Button>
            </CardHeader>
            <CardContent>
                <div className="space-y-4">
                    <div className="space-y-2">
                        <Label>Node Types</Label>
                        {['server', 'client', 'network'].map((type) => (
                            <div key={type} className="flex items-center space-x-2">
                                <Checkbox
                                    id={type}
                                    checked={selectedTypes.includes(type)}
                                    onCheckedChange={(checked) => {
                                        setSelectedTypes(prev => 
                                            checked
                                                ? [...prev, type]
                                                : prev.filter(t => t !== type)
                                        );
                                    }}
                                />
                                <Label htmlFor={type}>{type.charAt(0).toUpperCase() + type.slice(1)}</Label>
                            </div>
                        ))}
                    </div>
                    <div className="space-y-2">
                        <Label htmlFor="regexFilter">Regex Filter</Label>
                        <Input
                            id="regexFilter"
                            placeholder="Enter regex..."
                            value={regexFilter}
                            onChange={(e) => setRegexFilter(e.target.value)}
                        />
                    </div>
                    <Button onClick={applyFilters} className="w-full">
                        Apply Filters
                    </Button>
                </div>
            </CardContent>
        </Card>
    );
};

export default Sidebar;
