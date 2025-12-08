import React, { useEffect, useState } from 'react';
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import { Activity, Globe, Zap, Clock, Shield, AlertTriangle, CheckCircle, XCircle } from 'lucide-react';
import { cn } from '@/lib/utils';
import DashboardLayout from '@/layouts/DashboardLayout';
import LiveLogs from '@/components/LiveLogs';

const MetricCard = ({ title, value, icon: Icon, color }) => (
    <div className="glass-card flex flex-col gap-4">
        <div className="flex items-center justify-between">
            <span className="text-sm text-muted-foreground font-medium">{title}</span>
            <div className={`p-2 rounded-lg opacity-80 ${color || 'bg-primary/10 text-primary'}`}>
                <Icon className="w-4 h-4" />
            </div>
        </div>
        <div>
            <h3 className="text-2xl font-bold tracking-tight">{value}</h3>
        </div>
    </div>
);

export default function Dashboard() {
    const [metrics, setMetrics] = useState({
        total: 0,
        rps: 0,
        latency: 0,
        success: 0,
        authError: 0,
        clientError: 0,
        serverError: 0
    });
    const [data, setData] = useState([]);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        const fetchData = async () => {
            try {
                // 1. Fetch Metrics (Text)
                const resMetrics = await fetch('/observability/metrics', { cache: 'no-store' });
                const text = await resMetrics.text();
                // Parsing Logic
                let total = 0;
                let success = 0; // 2xx
                let authError = 0; // 401, 403
                let clientError = 0; // 4xx (except auth)
                let serverError = 0; // 5xx

                // Regex to match: drizzle_http_requests_total{method="GET",status="200",tenant_id="..."} 123
                const lines = text.split(/\r?\n/);
                lines.forEach(line => {
                    if (line.startsWith('drizzle_http_requests_total')) {
                        const match = line.match(/status="(\d+)"/);
                        const valMatch = line.match(/\s+(\d+)\s*$/);

                        if (match && valMatch) {
                            const status = parseInt(match[1]);
                            const count = parseInt(valMatch[1]);
                            total += count;

                            if (status >= 200 && status < 300) success += count;
                            else if (status === 401 || status === 403) authError += count;
                            else if (status >= 400 && status < 500) clientError += count;
                            else if (status >= 500) serverError += count;
                        }
                    }
                });

                setMetrics(prev => ({
                    total,
                    rps: Math.max(0, total - (prev.total || 0)), // Simple RPS calc
                    latency: 24, // Placeholder
                    success,
                    authError,
                    clientError,
                    serverError,
                    debug: {
                        linesCount: lines.length,
                        totalParsed: total,
                        sampleLine: lines.find(l => l.startsWith('drizzle_http_requests_total')) || "No match",
                        successCount: success,
                        error: null
                    }
                }));

                // 2. Fetch Stats (Historical)
                const resStats = await fetch('/observability/stats');
                if (resStats.ok) {
                    const statsJson = await resStats.json();
                    // Pivot: [{ time: '...', 200: 10, 429: 5, 401: 2, 400: 5, ... }]
                    const pivotted = {};
                    statsJson.forEach(item => {
                        const t = new Date(item.time).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
                        if (!pivotted[t]) pivotted[t] = { time: t, 200: 0, 400: 0, 401: 0, 429: 0, 500: 0 };
                        // Ensure we map the status groups correctly
                        pivotted[t][item.status] = (pivotted[t][item.status] || 0) + item.count;
                    });

                    // Convert to array and sort
                    const chartData = Object.values(pivotted).sort((a, b) => a.time.localeCompare(b.time));
                    if (chartData.length > 0) setData(chartData);
                }

            } catch (e) {
                console.error("Failed to fetch dashboard data", e);
                setMetrics(prev => ({
                    ...prev,
                    debug: { error: e.toString() }
                }));
            } finally {
                setLoading(false);
            }
        };

        fetchData();
        const interval = setInterval(fetchData, 2000);
        return () => clearInterval(interval);
    }, []);

    return (
        <DashboardLayout>
            <div className="space-y-8">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
                    <p className="text-muted-foreground mt-2">Real-time traffic analysis & security insights.</p>
                </div>

                <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                    <MetricCard title="Total Requests" value={metrics.total.toLocaleString()} icon={Globe} />
                    <MetricCard title="Requests / sec" value={metrics.rps} icon={Activity} color="bg-blue-500/10 text-blue-500" />

                    <MetricCard title="Success (2xx)" value={metrics.success.toLocaleString()} icon={CheckCircle} color="bg-green-500/10 text-green-500" />
                    <MetricCard title="Auth Errors (401/403)" value={metrics.authError.toLocaleString()} icon={Shield} color="bg-red-500/10 text-red-500" />
                    <MetricCard title="Client Errors (4xx)" value={metrics.clientError.toLocaleString()} icon={AlertTriangle} color="bg-yellow-500/10 text-yellow-500" />
                    <MetricCard title="Server Errors (5xx)" value={metrics.serverError.toLocaleString()} icon={XCircle} color="bg-orange-500/10 text-orange-500" />
                </div>

                <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                    <div className="glass-card h-[400px] lg:col-span-1">
                        <h3 className="text-lg font-semibold mb-6">Traffic Volume (Last 1h)</h3>
                        <ResponsiveContainer width="100%" height="90%">
                            <AreaChart data={data}>
                                <defs>
                                    <linearGradient id="colorSuccess" x1="0" y1="0" x2="0" y2="1">
                                        <stop offset="5%" stopColor="#22c55e" stopOpacity={0.3} />
                                        <stop offset="95%" stopColor="#22c55e" stopOpacity={0} />
                                    </linearGradient>
                                    <linearGradient id="colorAuth" x1="0" y1="0" x2="0" y2="1">
                                        <stop offset="5%" stopColor="#ef4444" stopOpacity={0.3} />
                                        <stop offset="95%" stopColor="#ef4444" stopOpacity={0} />
                                    </linearGradient>
                                    <linearGradient id="colorRateLimit" x1="0" y1="0" x2="0" y2="1">
                                        <stop offset="5%" stopColor="#a855f7" stopOpacity={0.3} />
                                        <stop offset="95%" stopColor="#a855f7" stopOpacity={0} />
                                    </linearGradient>
                                    <linearGradient id="colorClientError" x1="0" y1="0" x2="0" y2="1">
                                        <stop offset="5%" stopColor="#eab308" stopOpacity={0.3} />
                                        <stop offset="95%" stopColor="#eab308" stopOpacity={0} />
                                    </linearGradient>
                                    <linearGradient id="colorServerError" x1="0" y1="0" x2="0" y2="1">
                                        <stop offset="5%" stopColor="#f97316" stopOpacity={0.3} />
                                        <stop offset="95%" stopColor="#f97316" stopOpacity={0} />
                                    </linearGradient>
                                </defs>
                                <CartesianGrid strokeDasharray="3 3" stroke="hsl(var(--border))" vertical={false} />
                                <XAxis dataKey="time" hide />
                                <Tooltip
                                    contentStyle={{ backgroundColor: 'hsl(var(--card))', borderColor: 'hsl(var(--border))' }}
                                />
                                <Area type="monotone" dataKey="200" stackId="1" stroke="#22c55e" fill="url(#colorSuccess)" name="Success" />
                                <Area type="monotone" dataKey="400" stackId="1" stroke="#eab308" fill="url(#colorClientError)" name="Client Error" />
                                <Area type="monotone" dataKey="401" stackId="1" stroke="#ef4444" fill="url(#colorAuth)" name="Auth Error" />
                                <Area type="monotone" dataKey="429" stackId="1" stroke="#a855f7" fill="url(#colorRateLimit)" name="Rate Limited" />
                                <Area type="monotone" dataKey="500" stackId="1" stroke="#f97316" fill="url(#colorServerError)" name="Server Error" />
                            </AreaChart>
                        </ResponsiveContainer>
                    </div>

                    <div className="lg:col-span-2">
                        <LiveLogs />
                    </div>
                </div>
            </div>
        </DashboardLayout>
    );
}
