import React, { useEffect, useState } from 'react';
import { Server, Plus, ArrowRight } from 'lucide-react';
import DashboardLayout from '@/layouts/DashboardLayout';
import { cn } from '@/lib/utils';

export default function Services() {
    const [snapshot, setSnapshot] = useState({ services: [], routes: [] });
    const [showModal, setShowModal] = useState(false);

    // Flatten data
    const services = snapshot.services || [];
    const routes = snapshot.routes || [];

    useEffect(() => {
        fetch('/api/snapshot')
            .then(res => res.json())
            .then(data => setSnapshot(data))
            .catch(console.error);
    }, []);

    return (
        <DashboardLayout>
            <div className="flex justify-between items-center mb-8">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">Services</h1>
                    <p className="text-muted-foreground mt-2">Manage backend services and routing rules.</p>
                </div>
                <button className="bg-primary text-primary-foreground px-4 py-2 rounded-lg flex items-center gap-2 hover:opacity-90 transition-opacity">
                    <Plus className="w-4 h-4" />
                    Add Service
                </button>
            </div>

            <div className="space-y-6">
                {services.map(service => {
                    const serviceRoutes = routes.filter(r => r.service_id === service.id);

                    return (
                        <div key={service.id} className="glass-card">
                            <div className="flex items-center gap-4 mb-6 border-b border-border/50 pb-4">
                                <div className="p-2 bg-muted rounded-lg">
                                    <Server className="w-5 h-5" />
                                </div>
                                <div>
                                    <h3 className="font-semibold text-lg">{service.name}</h3>
                                    <div className="flex gap-2 text-xs text-muted-foreground mt-1">
                                        {service.urls.map(url => (
                                            <span key={url} className="px-2 py-0.5 bg-background rounded-full border border-border">{url}</span>
                                        ))}
                                    </div>
                                </div>
                            </div>

                            <div className="bg-background/30 rounded-lg p-4">
                                <h4 className="text-sm font-medium mb-3 text-muted-foreground">Routes</h4>
                                {serviceRoutes.length > 0 ? (
                                    <div className="space-y-2">
                                        {serviceRoutes.map(route => (
                                            <div key={route.id} className="flex items-center justify-between text-sm p-2 hover:bg-muted/50 rounded-md transition-colors">
                                                <div className="flex items-center gap-2">
                                                    <span className="font-medium">{route.name}</span>
                                                    <ArrowRight className="w-3 h-3 text-muted-foreground" />
                                                    <code className="text-primary bg-primary/10 px-1 py-0.5 rounded">{route.path_match.Prefix}</code>
                                                </div>
                                                <span className="text-xs text-muted-foreground font-mono">{route.id.split('-')[0]}</span>
                                            </div>
                                        ))}
                                    </div>
                                ) : (
                                    <p className="text-sm text-muted-foreground italic">No routes defined.</p>
                                )}
                            </div>
                        </div>
                    );
                })}
            </div>
        </DashboardLayout>
    );
}
