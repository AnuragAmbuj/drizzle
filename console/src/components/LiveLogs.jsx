import React, { useEffect, useState } from 'react';
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";

export default function LiveLogs() {
    const [logs, setLogs] = useState([]);

    useEffect(() => {
        const fetchLogs = async () => {
            try {
                // Direct fetch to gateway metrics port
                const res = await fetch('/observability/logs');
                if (!res.ok) return;
                const data = await res.json();
                setLogs(data);
            } catch (e) {
                console.error("Failed to fetch logs", e);
            }
        };

        fetchLogs();
        const interval = setInterval(fetchLogs, 2000); // 2s polling
        return () => clearInterval(interval);
    }, []);

    return (
        <Card className="col-span-1 lg:col-span-2">
            <CardHeader>
                <CardTitle>Live Requests (Last 100)</CardTitle>
            </CardHeader>
            <CardContent>
                <div className="overflow-auto h-[400px]">
                    <table className="w-full text-sm text-left">
                        <thead className="text-xs uppercase bg-secondary text-secondary-foreground sticky top-0">
                            <tr>
                                <th className="px-4 py-2">Time</th>
                                <th className="px-4 py-2">Method</th>
                                <th className="px-4 py-2">Path</th>
                                <th className="px-4 py-2">Status</th>
                                <th className="px-4 py-2">Latency (ms)</th>
                            </tr>
                        </thead>
                        <tbody>
                            {logs.map((log, i) => (
                                <tr key={i} className="border-b border-border hover:bg-muted/50">
                                    <td className="px-4 py-2 font-mono text-xs text-muted-foreground">
                                        {new Date(log.timestamp).toLocaleTimeString()}
                                    </td>
                                    <td className="px-4 py-2 font-bold">{log.method}</td>
                                    <td className="px-4 py-2 truncate max-w-[200px]" title={log.path}>{log.path}</td>
                                    <td className={`px-4 py-2 font-bold ${log.status >= 500 ? 'text-red-500' :
                                            log.status >= 400 ? 'text-yellow-500' :
                                                'text-green-500'
                                        }`}>
                                        {log.status}
                                    </td>
                                    <td className="px-4 py-2">{log.duration_ms.toFixed(2)}</td>
                                </tr>
                            ))}
                            {logs.length === 0 && (
                                <tr>
                                    <td colSpan={5} className="px-4 py-8 text-center text-muted-foreground">
                                        Waiting for traffic...
                                    </td>
                                </tr>
                            )}
                        </tbody>
                    </table>
                </div>
            </CardContent>
        </Card>
    );
}
