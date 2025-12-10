import React, { useEffect, useState } from 'react';
import { Users, Plus, Pencil, Trash2 } from 'lucide-react';
import DashboardLayout from '@/layouts/DashboardLayout';
import { useAuth } from '@/context/AuthContext';

export default function Tenants() {
    const [tenants, setTenants] = useState([]);
    const [isLoading, setIsLoading] = useState(true);
    const [showModal, setShowModal] = useState(false);
    const [showDeleteModal, setShowDeleteModal] = useState(false);
    const [isEditing, setIsEditing] = useState(false);
    const [currentId, setCurrentId] = useState(null);
    const [formData, setFormData] = useState({ slug: '', display_name: '' });
    const [tenantToDelete, setTenantToDelete] = useState(null);

    const fetchTenants = async () => {
        try {
            const res = await fetch('/api/snapshot');
            if (res.ok) {
                const snapshot = await res.json();
                setTenants(snapshot.tenants || []);
            }
        } catch (e) {
            console.error("Failed to fetch tenants", e);
        } finally {
            setIsLoading(false);
        }
    };

    useEffect(() => { fetchTenants(); }, []);

    const openCreate = () => {
        setIsEditing(false);
        setFormData({ slug: '', display_name: '' });
        setShowModal(true);
    };

    const openEdit = (tenant) => {
        setIsEditing(true);
        setCurrentId(tenant.id);
        setFormData({ slug: tenant.slug, display_name: tenant.display_name });
        setShowModal(true);
    };

    // Proper hook usage


    // Proper hook usage
    const { token } = useAuth();

    const handleFormSubmit = async (e) => {
        e.preventDefault();
        try {
            const url = isEditing ? `/api/tenants/${currentId}` : '/api/tenants';
            const method = isEditing ? 'PUT' : 'POST';

            const res = await fetch(url, {
                method,
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${token}`
                },
                body: JSON.stringify(formData)
            });

            if (res.ok) {
                setShowModal(false);
                setFormData({ slug: '', display_name: '' });
                fetchTenants();
            }
        } catch (e) {
            console.error("Failed to save tenant", e);
        }
    };

    const confirmDelete = async () => {
        if (!tenantToDelete) return;
        try {
            const res = await fetch(`/api/tenants/${tenantToDelete.id}`, {
                method: 'DELETE',
                headers: { 'Authorization': `Bearer ${token}` }
            });
            if (res.ok) {
                setShowDeleteModal(false);
                setTenantToDelete(null);
                fetchTenants();
            }
        } catch (e) {
            console.error("Failed to delete tenant", e);
        }
    };

    const handleSubmit = handleFormSubmit;

    return (
        <DashboardLayout>
            <div className="flex justify-between items-center mb-8">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">Tenants</h1>
                    <p className="text-muted-foreground mt-2">Manage your organizations and environments.</p>
                </div>
                <button onClick={openCreate} className="bg-primary text-primary-foreground px-4 py-2 rounded-lg flex items-center gap-2 hover:opacity-90 transition-opacity">
                    <Plus className="w-4 h-4" />
                    Create Tenant
                </button>
            </div>

            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                {tenants.map(tenant => (
                    <div key={tenant.id} className="glass-card flex flex-col gap-4 group hover:border-primary/50 transition-colors">
                        <div className="flex items-start justify-between">
                            <div className="flex items-center gap-3">
                                <div className="p-2 bg-muted rounded-lg group-hover:bg-primary/10 group-hover:text-primary transition-colors">
                                    <Users className="w-5 h-5" />
                                </div>
                                <div>
                                    <h3 className="font-semibold text-lg">{tenant.display_name}</h3>
                                    <p className="text-sm text-muted-foreground">/{tenant.slug}</p>
                                </div>
                            </div>
                            <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                                <button
                                    onClick={(e) => { e.stopPropagation(); openEdit(tenant); }}
                                    className="p-2 hover:bg-muted rounded-md text-muted-foreground hover:text-foreground"
                                    title="Edit"
                                >
                                    <Pencil className="w-4 h-4" />
                                </button>
                                <button
                                    onClick={(e) => { e.stopPropagation(); setTenantToDelete(tenant); setShowDeleteModal(true); }}
                                    className="p-2 hover:bg-destructive/10 rounded-md text-muted-foreground hover:text-destructive"
                                    title="Delete"
                                >
                                    <Trash2 className="w-4 h-4" />
                                </button>
                            </div>
                        </div>
                        <div className="text-xs text-muted-foreground font-mono mt-auto">ID: {tenant.id}</div>
                    </div>
                ))}
            </div>

            {tenants.length === 0 && !isLoading && (
                <div className="text-center py-20 text-muted-foreground">
                    No tenants found. Create one to get started.
                </div>
            )}

            {showModal && (
                <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4">
                    <div className="bg-card w-full max-w-md p-6 rounded-xl border border-border shadow-2xl">
                        <h2 className="text-xl font-bold mb-4">{isEditing ? 'Edit Tenant' : 'Create New Tenant'}</h2>
                        <form onSubmit={handleSubmit} className="space-y-4">
                            <div>
                                <label className="text-sm font-medium mb-1 block">Display Name</label>
                                <input
                                    type="text"
                                    required
                                    className="w-full bg-background border border-input rounded-md px-3 py-2 focus:ring-2 focus:ring-primary focus:outline-none"
                                    value={formData.display_name}
                                    onChange={e => setFormData({ ...formData, display_name: e.target.value })}
                                />
                            </div>
                            <div>
                                <label className="text-sm font-medium mb-1 block">Slug</label>
                                <input
                                    type="text"
                                    required
                                    disabled={isEditing}
                                    className={`w-full bg-background border border-input rounded-md px-3 py-2 focus:ring-2 focus:ring-primary focus:outline-none ${isEditing ? 'opacity-50 cursor-not-allowed' : ''}`}
                                    value={formData.slug}
                                    onChange={e => setFormData({ ...formData, slug: e.target.value })}
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
                        <h2 className="text-xl font-bold mb-2 text-destructive">Delete Tenant</h2>
                        <p className="text-muted-foreground mb-6">
                            Are you sure you want to delete <span className="font-semibold text-foreground">{tenantToDelete?.display_name}</span>? This action cannot be undone.
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
