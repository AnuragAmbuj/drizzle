import React, { useEffect, useState, useRef } from 'react';
import DashboardLayout from '@/layouts/DashboardLayout';
import { Terminal, Pause, Play, Trash2 } from 'lucide-react';

const LiveLogs = () => {
    const [logs, setLogs] = useState([]);
    const [isPaused, setIsPaused] = useState(false);
    const endRef = useRef(null);
    const eventSourceRef = useRef(null);

    useEffect(() => {
        if (isPaused) return;

        // Use vite proxy path
        const eventSource = new EventSource('/observability/logs/live');
        eventSourceRef.current = eventSource;

        eventSource.onmessage = (event) => {
            setLogs(prev => {
                const newLogs = [...prev, event.data];
                if (newLogs.length > 500) return newLogs.slice(-500); // Keep last 500
                return newLogs;
            });
        };

        eventSource.onerror = (e) => {
            console.error("EventSource failed:", e);
            eventSource.close();
            // Retry logic could go here
        };

        return () => {
            eventSource.close();
        };
    }, [isPaused]);

    useEffect(() => {
        if (!isPaused && endRef.current) {
            endRef.current.scrollIntoView({ behavior: "smooth" });
        }
    }, [logs, isPaused]);

    const togglePause = () => {
        if (isPaused) {
            setIsPaused(false);
        } else {
            setIsPaused(true);
            if (eventSourceRef.current) {
                eventSourceRef.current.close();
            }
        }
    };

    const clearLogs = () => setLogs([]);

    return (
        <DashboardLayout>
            <div className="flex justify-between items-center mb-6">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">Live Logs</h1>
                    <p className="text-muted-foreground mt-2">Real-time streaming logs from the gateway.</p>
                </div>
                <div className="flex gap-2">
                    <button onClick={togglePause} className="bg-secondary text-secondary-foreground px-4 py-2 rounded-lg flex items-center gap-2 hover:opacity-90">
                        {isPaused ? <Play className="w-4 h-4" /> : <Pause className="w-4 h-4" />}
                        {isPaused ? "Resume" : "Pause"}
                    </button>
                    <button onClick={clearLogs} className="bg-destructive text-destructive-foreground px-4 py-2 rounded-lg flex items-center gap-2 hover:opacity-90">
                        <Trash2 className="w-4 h-4" />
                        Clear
                    </button>
                </div>
            </div>

            <div className="bg-black text-green-400 p-4 rounded-lg font-mono text-sm h-[600px] overflow-y-auto border border-gray-800 shadow-inner">
                {logs.length === 0 && <div className="text-gray-500 italic">Waiting for logs...</div>}
                {logs.map((log, i) => (
                    <div key={i} className="whitespace-pre-wrap break-all border-b border-gray-900/50 py-0.5 hover:bg-white/5">
                        {log}
                    </div>
                ))}
                <div ref={endRef} />
            </div>
        </DashboardLayout>
    );
};

export default LiveLogs;
