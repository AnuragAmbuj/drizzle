import React, { useEffect, useState } from 'react';
import { Shield, Key, FileCode, Plus, Pencil, Trash2, Lock } from 'lucide-react';
import DashboardLayout from '@/layouts/DashboardLayout';
import { useAuth } from '@/context/AuthContext';

export default function Policies() {
    const [snapshot, setSnapshot] = useState({ limits: [], policies: [], api_keys: {}, tenants: [] });
    const { token } = useAuth();

    // UI State
    const [modalType, setModalType] = useState(null); // 'limit', 'policy', 'apikey'
    const [isEditing, setIsEditing] = useState(false);
    const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
    const [selectedItem, setSelectedItem] = useState(null); // Item to edit/delete

    // Security Config State
    const [securityConfig, setSecurityConfig] = useState({ global_rate_limit: 100, global_burst: 50 });
    const [isSavingSecurity, setIsSavingSecurity] = useState(false);
    const [securityMsg, setSecurityMsg] = useState(null);

    // ... Form State ...

    // Form State
    const [formData, setFormData] = useState({});

    const fetchSnapshot = async () => {
        try {
            const [snapRes, secRes] = await Promise.all([
                fetch('/api/snapshot'),
                fetch('/api/security')
            ]);

            if (snapRes.ok) setSnapshot(await snapRes.json());
            if (secRes.ok) setSecurityConfig(await secRes.json());
        } catch (e) { console.error(e); }
    };

    useEffect(() => { fetchSnapshot(); }, []);

    // --- Handlers ---

    const openCreate = (type) => {
        setModalType(type);
        setIsEditing(false);
        setFormData({ tenant_id: snapshot.tenants[0]?.id || '' });
    };

    const openEdit = (type, item) => {
        setModalType(type);
        setIsEditing(true);
        setSelectedItem(item);
        setFormData({ ...item });
    };

    const openDelete = (type, item) => {
        setModalType(type);
        setSelectedItem(item);
        setShowDeleteConfirm(true);
    };

    const handleSubmit = async (e) => {
        e.preventDefault();
        let url = '';
        let method = isEditing ? 'PUT' : 'POST';
        let body = { ...formData }; // Clone

        // Determine URL and payload adjustments
        if (modalType === 'limit') {
            url = isEditing ? `/api/limits/${selectedItem.id}` : '/api/limits'; // NOTE: Update limit not yet in API? 
            // Wait, I implemented `create_limit_policy` but did I implement `update_limit_policy`?
            // Checking my memory/past steps. I implemented `update_policy` (OPA) but maybe not `update_limit`?
            // Step 3091: `route("/limits", post(create_limit_policy))`
            // I DID NOT implement update/delete for limits in `admin-api/src/main.rs`.
            // I only implemented `create_limit_policy`. 
            // I implemented `update_policy` (OPA).
            // I implemented `create_api_key`. No update/delete for API Keys?
            // Step 3091: `route("/api-keys", post(create_api_key))`
            // So I am missing Update/Delete endpoints for Limits and Keys in `admin-api`?
            // I implemented `PolicyRepository` (OPA) CRUD.
            // I implemented `Tenant` and `Service` CRUD.
            // I likely missed `LimitPolicy` and `ApiKey` Update/Delete in `admin-api`.
            // I will only implement Create for Limits/Keys if I can't update them, or I must go back to backend.
            // The user Step 3087 Checkpoint says: "Implement API Key and Limit Policy CRUD (Next Steps)".
            // So I haven't done it yet.
            // I should simply display them and maybe support "Create" only for now, or "Delete" if I add the endpoint.
            // I will stick to what I have: `update_policy` (OPA) exists. `delete_policy` (OPA) exists.
            // `create_limit` exists.
            // `create_api_key` exists.
            // I will disable Edit/Delete for Limits/Keys for now in the UI or implement them in Backend first?
            // The user objective is "Admin CRUD". I should probably implement them.
            // But for this step "Enhance Console UI", I will implement UI for what EXISTS.
            // I will Add UI for `policies` (OPA) which HAS CRUD.
            // I will Add Create UI for Limits/Keys.
        } else if (modalType === 'policy') {
            url = isEditing ? `/api/policies/${selectedItem.id}` : '/api/policies';
        } else if (modalType === 'apikey') {
            url = '/api/api-keys';
            method = 'POST'; // No edit for keys usually
        }

        if (!url) return;

        // Fix number types for limits
        if (modalType === 'limit') {
            body.rate = parseInt(body.rate);
            body.burst = parseInt(body.burst);
        }

        try {
            const res = await fetch(url, {
                method,
                headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${token}` },
                body: JSON.stringify(body)
            });
            if (res.ok) {
                setModalType(null);
                fetchSnapshot();
            }
        } catch (e) { console.error(e); }
    };

    const confirmDelete = async () => {
        if (!selectedItem || !modalType) return;
        let url = '';
        if (modalType === 'policy') url = `/api/policies/${selectedItem.id}`;
        // if (modalType === 'limit') url = `/api/limits/${selectedItem.id}`; // Not implemented handling
        // if (modalType === 'apikey') url = `/api/api-keys/${selectedItem.key}`; // Not implemented

        if (!url) return;

        try {
            const res = await fetch(url, {
                method: 'DELETE',
                headers: { 'Authorization': `Bearer ${token}` }
            });
            if (res.ok) {
                setShowDeleteConfirm(false);
                setSelectedItem(null);
                fetchSnapshot();
            }
        } catch (e) { console.error(e); }
    };

    return (
        <DashboardLayout>
            <div className="mb-8">
                <h1 className="text-3xl font-bold tracking-tight">Policies</h1>
                <p className="text-muted-foreground mt-2">Zero-Trust Configuration & Rate Limiting.</p>
            </div>

            <div className="grid gap-8 lg:grid-cols-2">

                {/* Global Security Settings - NEW */}
                <div className="glass-card md:col-span-2">
                    <div className="flex items-center gap-3 mb-4">
                        <Lock className="w-5 h-5 text-primary" />
                        <div>
                            <h2 className="text-xl font-bold">DoS Protection</h2>
                            <p className="text-xs text-muted-foreground">Global Rate Limiting (Token Bucket)</p>
                        </div>
                    </div>
                    <div className="flex items-end gap-4">
                        <div className="flex-1 max-w-xs">
                            <label className="text-xs font-semibold uppercase text-muted-foreground mb-1 block">Requests / Sec</label>
                            <input
                                type="number"
                                className="w-full bg-background border border-input rounded-md px-3 py-2"
                                value={securityConfig.global_rate_limit}
                                onChange={e => setSecurityConfig({ ...securityConfig, global_rate_limit: parseInt(e.target.value) || 0 })}
                            />
                        </div>
                        <div className="flex-1 max-w-xs">
                            <label className="text-xs font-semibold uppercase text-muted-foreground mb-1 block">Burst</label>
                            <input
                                type="number"
                                className="w-full bg-background border border-input rounded-md px-3 py-2"
                                value={securityConfig.global_burst}
                                onChange={e => setSecurityConfig({ ...securityConfig, global_burst: parseInt(e.target.value) || 0 })}
                            />
                        </div>
                        <button
                            onClick={saveSecurity}
                            disabled={isSavingSecurity}
                            className="bg-primary text-primary-foreground px-4 py-2 rounded-md font-medium disabled:opacity-50 h-10"
                        >
                            {isSavingSecurity ? 'Saving...' : 'Update Config'}
                        </button>
                        {securityMsg && (
                            <span className={`text-sm font-medium self-center ${securityMsg.type === 'success' ? 'text-green-500' : 'text-red-500'}`}>
                                {securityMsg.text}
                            </span>
                        )}
                    </div>
                </div>

                {/* OPA Policies - FULL CRUD */}
                <div className="glass-card">
                    <div className="flex items-center justify-between mb-6">
                        <div className="flex items-center gap-3">
                            <FileCode className="w-5 h-5 text-primary" />
                            <h2 className="text-xl font-bold">Access Policies</h2>
                        </div>
                        <button onClick={() => openCreate('policy')} className="p-2 hover:bg-muted rounded-full">
                            <Plus className="w-4 h-4" />
                        </button>
                    </div>
                    <div className="space-y-3">
                        {snapshot.policies?.map(p => (
                            <div key={p.id} className="p-3 border border-border rounded-lg bg-background/50 flex justify-between items-start group">
                                <div>
                                    <div className="font-medium">{p.name}</div>
                                    <div className="text-xs text-muted-foreground font-mono mt-1">Tenant: {p.tenant_id}</div>
                                </div>
                                <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                                    <button onClick={() => openEdit('policy', p)} className="p-1.5 hover:bg-muted rounded text-muted-foreground">
                                        <Pencil className="w-3.5 h-3.5" />
                                    </button>
                                    <button onClick={() => openDelete('policy', p)} className="p-1.5 hover:bg-destructive/10 rounded text-destructive">
                                        <Trash2 className="w-3.5 h-3.5" />
                                    </button>
                                </div>
                            </div>
                        ))}
                    </div>
                </div>

                {/* Rate Limits - CREATE READ ONLY */}
                <div className="glass-card">
                    <div className="flex items-center justify-between mb-6">
                        <div className="flex items-center gap-3">
                            <Shield className="w-5 h-5 text-primary" />
                            <h2 className="text-xl font-bold">Rate Limits</h2>
                        </div>
                        <button onClick={() => openCreate('limit')} className="p-2 hover:bg-muted rounded-full">
                            <Plus className="w-4 h-4" />
                        </button>
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
                    </div>
                </div>

                {/* API Keys - CREATE READ ONLY */}
                <div className="glass-card md:col-span-2 lg:col-span-1">
                    <div className="flex items-center justify-between mb-6">
                        <div className="flex items-center gap-3">
                            <Key className="w-5 h-5 text-primary" />
                            <h2 className="text-xl font-bold">API Keys</h2>
                        </div>
                        <button onClick={() => openCreate('apikey')} className="p-2 hover:bg-muted rounded-full">
                            <Plus className="w-4 h-4" />
                        </button>
                    </div>
                    <div className="space-y-3">
                        {Object.entries(snapshot.api_keys || {}).map(([key, tenant_id]) => (
                            <div key={key} className="p-3 border border-border rounded-lg bg-background/50 flex justify-between items-center">
                                <code className="text-sm font-mono text-primary">{key}</code>
                                <span className="text-xs text-muted-foreground font-mono">...{tenant_id.slice(-6)}</span>
                            </div>
                        ))}
                    </div>
                </div>
            </div>

            {/* Universal Modal */}
            {modalType && !showDeleteConfirm && (
                <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4">
                    <div className="bg-card w-full max-w-md p-6 rounded-xl border border-border shadow-2xl">
                        <h2 className="text-xl font-bold mb-4">
                            {isEditing ? 'Edit' : 'Create'} {modalType === 'policy' ? 'Policy' : modalType === 'limit' ? 'Rate Limit' : 'API Key'}
                        </h2>
                        <form onSubmit={handleSubmit} className="space-y-4">
                            {/* Tenant Select - Common */}
                            {!isEditing && (
                                <div>
                                    <label className="text-sm font-medium mb-1 block">Tenant</label>
                                    <select
                                        className="w-full bg-background border border-input rounded-md px-3 py-2"
                                        value={formData.tenant_id}
                                        onChange={e => setFormData({ ...formData, tenant_id: e.target.value })}
                                        required
                                    >
                                        <option value="">Select Tenant</option>
                                        {snapshot.tenants.map(t => <option key={t.id} value={t.id}>{t.display_name}</option>)}
                                    </select>
                                </div>
                            )}

                            {/* Policy Fields */}
                            {modalType === 'policy' && (
                                <>
                                    <div>
                                        <label className="text-sm font-medium mb-1 block">Name</label>
                                        <input type="text" required className="w-full bg-background border border-input rounded-md px-3 py-2"
                                            value={formData.name || ''}
                                            onChange={e => setFormData({ ...formData, name: e.target.value })}
                                        />
                                    </div>
                                    <div>
                                        <label className="text-sm font-medium mb-1 block">Rego Content</label>
                                        <textarea required className="w-full bg-background border border-input rounded-md px-3 py-2 h-32 font-mono text-xs"
                                            value={formData.content || ''}
                                            onChange={e => setFormData({ ...formData, content: e.target.value })}
                                            placeholder="package example..."
                                        />
                                    </div>
                                </>
                            )}

                            {/* Limit Fields */}
                            {modalType === 'limit' && (
                                <>
                                    <div>
                                        <label className="text-sm font-medium mb-1 block">Name</label>
                                        <input type="text" required className="w-full bg-background border border-input rounded-md px-3 py-2"
                                            value={formData.name || ''}
                                            onChange={e => setFormData({ ...formData, name: e.target.value })}
                                        />
                                    </div>
                                    <div className="grid grid-cols-2 gap-4">
                                        <div>
                                            <label className="text-sm font-medium mb-1 block">Rate (rps)</label>
                                            <input type="number" required className="w-full bg-background border border-input rounded-md px-3 py-2"
                                                value={formData.rate || ''}
                                                onChange={e => setFormData({ ...formData, rate: e.target.value })}
                                            />
                                        </div>
                                        <div>
                                            <label className="text-sm font-medium mb-1 block">Burst</label>
                                            <input type="number" required className="w-full bg-background border border-input rounded-md px-3 py-2"
                                                value={formData.burst || ''}
                                                onChange={e => setFormData({ ...formData, burst: e.target.value })}
                                            />
                                        </div>
                                    </div>
                                </>
                            )}

                            {/* API Key Fields */}
                            {modalType === 'apikey' && (
                                <div>
                                    <label className="text-sm font-medium mb-1 block">Key Value</label>
                                    <input type="text" required className="w-full bg-background border border-input rounded-md px-3 py-2 font-mono"
                                        value={formData.key_value || ''}
                                        onChange={e => setFormData({ ...formData, key_value: e.target.value })}
                                    />
                                </div>
                            )}

                            <div className="flex justify-end gap-3 pt-4">
                                <button type="button" onClick={() => setModalType(null)} className="px-4 py-2 text-sm hover:underline">Cancel</button>
                                <button type="submit" className="bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm font-medium">
                                    {isEditing ? 'Save Changes' : 'Create'}
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            )}

            {showDeleteConfirm && (
                <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4">
                    <div className="bg-card w-full max-w-md p-6 rounded-xl border border-border shadow-2xl">
                        <h2 className="text-xl font-bold mb-2 text-destructive">Delete Policy</h2>
                        <p className="text-muted-foreground mb-6">Are you sure?</p>
                        <div className="flex justify-end gap-3">
                            <button onClick={() => setShowDeleteConfirm(false)} className="px-4 py-2 text-sm hover:underline">Cancel</button>
                            <button onClick={confirmDelete} className="bg-destructive text-destructive-foreground px-4 py-2 rounded-md text-sm font-medium">Delete</button>
                        </div>
                    </div>
                </div>
            )}
        </DashboardLayout>
    );
}
