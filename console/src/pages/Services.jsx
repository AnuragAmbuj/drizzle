import React, { useEffect, useState } from 'react';
import { Server, Plus, ArrowRight, Pencil, Trash2 } from 'lucide-react';
import DashboardLayout from '@/layouts/DashboardLayout';
import { useAuth } from '@/context/AuthContext';

export default function Services() {
    const [snapshot, setSnapshot] = useState({ services: [], routes: [], tenants: [] });
    const [showModal, setShowModal] = useState(false);
    const [showDeleteModal, setShowDeleteModal] = useState(false);
    const [isEditing, setIsEditing] = useState(false);
    const [currentId, setCurrentId] = useState(null);
    const [formData, setFormData] = useState({ name: '', host: '', tenant_id: '' });
    const [serviceToDelete, setServiceToDelete] = useState(null);
    const { token } = useAuth();

    // Flatten data
    const services = snapshot.services || [];
    const routes = snapshot.routes || [];
    const tenants = snapshot.tenants || [];

    const fetchSnapshot = async () => {
        try {
            const res = await fetch('/api/snapshot');
            if (res.ok) {
                setSnapshot(await res.json());
            }
        } catch (e) { console.error(e); }
    };

    useEffect(() => { fetchSnapshot(); }, []);

    const openCreate = () => {
        setIsEditing(false);
        setFormData({ name: '', host: '', tenant_id: tenants[0]?.id || '' });
        setShowModal(true);
    };

    const openEdit = (service) => {
        setIsEditing(true);
        setCurrentId(service.id);
        // Join hosts for display if multiple, currently backend create takes single.
        setFormData({
            name: service.name,
            host: (service.hosts || []).join(', '),
            tenant_id: service.tenant_id
        });
        setShowModal(true);
    };

    const handleSubmit = async (e) => {
        e.preventDefault();
        try {
            const url = isEditing ? `/api/services/${currentId}` : '/api/services';
            const method = isEditing ? 'PUT' : 'POST';

            // For update, we send hosts array. For create, we send single host.
            // Backend `CreateServiceRequest` has `host: String`.
            // Backend `UpdateServiceRequest` has `hosts: Vec<String>`.

            const payload = isEditing ? {
                name: formData.name,
                hosts: formData.host.split(',').map(h => h.trim()).filter(h => h)
            } : {
                name: formData.name,
                host: formData.host,
                tenant_id: formData.tenant_id
            };

            const res = await fetch(url, {
                method,
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${token}`
                },
                body: JSON.stringify(payload)
            });

            if (res.ok) {
                setShowModal(false);
                setFormData({ name: '', host: '', tenant_id: '' });
                fetchSnapshot();
            }
        } catch (e) {
            console.error("Failed to save service", e);
        }
    };

    const confirmDelete = async () => {
        if (!serviceToDelete) return;
        try {
            const res = await fetch(`/api/services/${serviceToDelete.id}`, {
                method: 'DELETE',
                headers: { 'Authorization': `Bearer ${token}` }
            });
            if (res.ok) {
                setShowDeleteModal(false);
                setServiceToDelete(null);
                fetchSnapshot();
            }
        } catch (e) {
            console.error("Failed to delete service", e);
        }
    };

    return (
        <DashboardLayout>
            <div className="flex justify-between items-center mb-8">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">Services</h1>
                    <p className="text-muted-foreground mt-2">Manage backend services and routing rules.</p>
                </div>
                <button onClick={openCreate} className="bg-primary text-primary-foreground px-4 py-2 rounded-lg flex items-center gap-2 hover:opacity-90 transition-opacity">
                    <Plus className="w-4 h-4" />
                    Add Service
                </button>
            </div>

            <div className="space-y-6">
                {services.map(service => {
                    const serviceRoutes = routes.filter(r => r.service_id === service.id);

                    return (
                        <div key={service.id} className="glass-card">
                            <div className="flex items-center gap-4 mb-6 border-b border-border/50 pb-4 justify-between">
                                <div className="flex items-center gap-4">
                                    <div className="p-2 bg-muted rounded-lg">
                                        <Server className="w-5 h-5" />
                                    </div>
                                    <div>
                                        <h3 className="font-semibold text-lg">{service.name}</h3>
                                        <div className="flex gap-2 text-xs text-muted-foreground mt-1">
                                            {(service.hosts || []).map(url => (
                                                <span key={url} className="px-2 py-0.5 bg-background rounded-full border border-border">{url}</span>
                                            ))}
                                        </div>
                                    </div>
                                </div>
                                <div className="flex gap-2">
                                    <button
                                        onClick={() => openEdit(service)}
                                        className="p-2 hover:bg-muted rounded-md text-muted-foreground hover:text-foreground"
                                        title="Edit"
                                    >
                                        <Pencil className="w-4 h-4" />
                                    </button>
                                    <button
                                        onClick={() => { setServiceToDelete(service); setShowDeleteModal(true); }}
                                        className="p-2 hover:bg-destructive/10 rounded-md text-muted-foreground hover:text-destructive"
                                        title="Delete"
                                    >
                                        <Trash2 className="w-4 h-4" />
                                    </button>
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
                                                    <code className="text-primary bg-primary/10 px-1 py-0.5 rounded">
                                                        {route.match_path?.value || route.match_path?.Prefix || '/'}
                                                    </code>
                                                </div>
                                                <div className="flex items-center gap-4">
                                                    <span className="text-xs text-muted-foreground font-mono">{route.id.split('-')[0]}</span>
                                                    {/* Route Delete button could go here too? MVP: Just Service level */}
                                                </div>
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

            {/* Modal */}
            {showModal && (
                <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4">
                    <div className="bg-card w-full max-w-md p-6 rounded-xl border border-border shadow-2xl">
                        <h2 className="text-xl font-bold mb-4">{isEditing ? 'Edit Service' : 'Add Service'}</h2>
                        <form onSubmit={handleSubmit} className="space-y-4">
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
                                        {tenants.map(t => <option key={t.id} value={t.id}>{t.display_name}</option>)}
                                    </select>
                                </div>
                            )}
                            <div>
                                <label className="text-sm font-medium mb-1 block">Service Name</label>
                                <input
                                    type="text"
                                    required
                                    className="w-full bg-background border border-input rounded-md px-3 py-2"
                                    value={formData.name}
                                    onChange={e => setFormData({ ...formData, name: e.target.value })}
                                />
                            </div>
                            <div>
                                <label className="text-sm font-medium mb-1 block">Host(s) {isEditing && <span className="text-xs text-muted-foreground">(comma separated)</span>}</label>
                                <input
                                    type="text"
                                    required
                                    className="w-full bg-background border border-input rounded-md px-3 py-2"
                                    value={formData.host}
                                    onChange={e => setFormData({ ...formData, host: e.target.value })}
                                    placeholder="e.g. backend:8080"
                                />
                            </div>
                            <div className="flex justify-end gap-3 pt-4">
                                <button type="button" onClick={() => setShowModal(false)} className="px-4 py-2 text-sm hover:underline">Cancel</button>
                                <button type="submit" className="bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm font-medium">
                                    {isEditing ? 'Save Changes' : 'Create'}
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            )}

            {showDeleteModal && (
                <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4">
                    <div className="bg-card w-full max-w-md p-6 rounded-xl border border-border shadow-2xl">
                        <h2 className="text-xl font-bold mb-2 text-destructive">Delete Service</h2>
                        <p className="text-muted-foreground mb-6">
                            Are you sure you want to delete <span className="font-semibold text-foreground">{serviceToDelete?.name}</span>?
                            Also deletes associated routes.
                        </p>
                        <div className="flex justify-end gap-3">
                            <button onClick={() => setShowDeleteModal(false)} className="px-4 py-2 text-sm hover:underline">Cancel</button>
                            <button onClick={confirmDelete} className="bg-destructive text-destructive-foreground px-4 py-2 rounded-md text-sm font-medium">Delete</button>
                        </div>
                    </div>
                </div>
            )}
        </DashboardLayout>
    );
}
