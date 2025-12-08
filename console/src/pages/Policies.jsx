import React, { useEffect, useState } from 'react';
import { Shield, Key } from 'lucide-react';
import DashboardLayout from '@/layouts/DashboardLayout';

export default function Policies() {
    const [snapshot, setSnapshot] = useState({ limits: [], policies: [], api_keys: {} });

    useEffect(() => {
        fetch('/api/snapshot')
            .then(res => res.json())
            .then(data => setSnapshot(data))
            .catch(console.error);
    }, []);

    return (
        <DashboardLayout>
            <div className="mb-8">
                <h1 className="text-3xl font-bold tracking-tight">Policies</h1>
                <p className="text-muted-foreground mt-2">Zero-Trust Configuration.</p>
            </div>

            <div className="grid gap-8 md:grid-cols-2">
                {/* Rate Limits */}
                <div className="glass-card">
                    <div className="flex items-center gap-3 mb-6">
                        <Shield className="w-5 h-5 text-primary" />
                        <h2 className="text-xl font-bold">Rate Limits</h2>
                    </div>
                    <div className="space-y-3">
                        {snapshot.limits?.map(limit => (
                            <div key={limit.id} className="p-3 border border-border rounded-lg bg-background/50">
                                <div className="flex justify-between items-center mb-2">
                                    <span className="font-medium">{limit.name}</span>
                                    <span className="text-xs bg-primary/20 text-primary px-2 py-1 rounded">
                                        {limit.rate} rps / {limit.burst} burst
                                    </span>
                                </div>
                                <div className="text-xs text-muted-foreground font-mono truncate">
                                    Tenant: {limit.tenant_id}
                                </div>
                            </div>
                        ))}
                        {(!snapshot.limits || snapshot.limits.length === 0) && <p className="text-muted-foreground text-sm">No limits defined.</p>}
                    </div>
                </div>

                {/* API Keys */}
                <div className="glass-card">
                    <div className="flex items-center gap-3 mb-6">
                        <Key className="w-5 h-5 text-primary" />
                        <h2 className="text-xl font-bold">API Keys</h2>
                    </div>
                    <div className="space-y-3">
                        {Object.entries(snapshot.api_keys || {}).map(([key, tenant_id]) => (
                            <div key={key} className="p-3 border border-border rounded-lg bg-background/50 flex justify-between items-center">
                                <code className="text-sm font-mono text-primary">{key}</code>
                                <span className="text-xs text-muted-foreground font-mono">...{tenant_id.slice(-6)}</span>
                            </div>
                        ))}
                        {(!snapshot.api_keys || Object.keys(snapshot.api_keys).length === 0) && <p className="text-muted-foreground text-sm">No API keys found.</p>}
                    </div>
                </div>
            </div>
        </DashboardLayout>
    );
}
