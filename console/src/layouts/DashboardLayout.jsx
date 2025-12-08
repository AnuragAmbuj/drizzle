import { Link, useLocation } from 'react-router-dom';
import { LayoutDashboard, Users, Server, Shield, Activity } from 'lucide-react';
import { cn } from '@/lib/utils';
import { useEffect, useState } from 'react';

const SidebarItem = ({ icon: Icon, label, href }) => {
    const location = useLocation();
    const isActive = location.pathname === href;

    return (
        <Link
            to={href}
            className={cn(
                "flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all duration-200 group text-sm font-medium",
                isActive
                    ? "bg-primary/10 text-primary"
                    : "text-muted-foreground hover:bg-muted/50 hover:text-foreground"
            )}
        >
            <Icon className={cn("w-4 h-4", isActive ? "text-primary" : "text-muted-foreground group-hover:text-foreground")} />
            {label}
        </Link>
    );
};

export default function DashboardLayout({ children }) {
    const [health, setHealth] = useState("Checking...");

    // Poll for health status
    useEffect(() => {
        const checkHealth = async () => {
            try {
                // We proxy /observability -> localhost:9111
                const res = await fetch('/observability/health');
                if (res.ok) setHealth("Online");
                else setHealth("Degraded");
            } catch (e) {
                setHealth("Offline");
            }
        };
        checkHealth();
        const interval = setInterval(checkHealth, 5000);
        return () => clearInterval(interval);
    }, []);

    return (
        <div className="flex h-screen bg-background text-foreground overflow-hidden">
            {/* Sidebar */}
            <aside className="w-64 border-r border-border bg-card/50 backdrop-blur-xl hidden md:flex flex-col">
                <div className="h-16 flex items-center px-6 border-b border-border">
                    <div className="flex items-center gap-2">
                        <div className="w-6 h-6 bg-primary rounded-md flex items-center justify-center">
                            <span className="text-primary-foreground font-bold">D</span>
                        </div>
                        <span className="font-bold text-lg tracking-tight">Drizzle</span>
                    </div>
                </div>

                <div className="flex-1 px-4 py-6 space-y-1">
                    <SidebarItem icon={LayoutDashboard} label="Dashboard" href="/" />
                    <SidebarItem icon={Users} label="Tenants" href="/tenants" />
                    <SidebarItem icon={Server} label="Services" href="/services" />
                    <SidebarItem icon={Shield} label="Policies" href="/policies" />
                </div>

                <div className="p-4 border-t border-border">
                    <div className="flex items-center gap-3 px-3 py-2 rounded-lg bg-muted/20 border border-border/50">
                        <Activity className={cn("w-4 h-4", health === "Online" ? "text-green-500" : "text-red-500")} />
                        <div className="flex flex-col">
                            <span className="text-xs text-muted-foreground">Gateway Status</span>
                            <span className="text-sm font-semibold">{health}</span>
                        </div>
                    </div>
                </div>
            </aside>

            {/* Main Content */}
            <main className="flex-1 flex flex-col overflow-auto relative">
                <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-primary/5 via-background to-background pointer-events-none" />
                <div className="relative z-10 p-8">
                    {children}
                </div>
            </main>
        </div>
    );
}
