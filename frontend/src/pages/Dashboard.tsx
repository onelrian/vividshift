import { motion } from "framer-motion";
import {
    BarChart3,
    Calendar,
    Users,
} from "lucide-react";
import { useState, useEffect } from "react";
import { useAuth } from "../context/AuthContext";
import { Layout } from "../components/layout/Layout";
import type { DashboardData } from "../types";

export const Dashboard = () => {
    const { session } = useAuth();
    const [data, setData] = useState<DashboardData | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [shuffling, setShuffling] = useState(false);

    const fetchData = async () => {
        try {
            setLoading(true);
            const resp = await fetch('/api/dashboard', {
                headers: {
                    'Authorization': `Bearer ${session?.access_token}`,
                }
            });
            if (!resp.ok) throw new Error('Failed to fetch dashboard data');
            const json = await resp.json();
            setData(json);
        } catch (err) {
            setError(err instanceof Error ? err.message : 'An error occurred');
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        if (session) fetchData();
    }, [session]);

    const triggerShuffle = async () => {
        if (!window.confirm("Are you sure you want to trigger a manual shuffle? This will reassign roles immediately.")) return;

        try {
            setShuffling(true);
            const resp = await fetch('/api/shuffle', {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${session?.access_token}`,
                }
            });
            if (!resp.ok) throw new Error('Failed to trigger shuffle');
            alert("Shuffle triggered successfully!");
            fetchData(); // Refresh data
        } catch (err) {
            alert(err instanceof Error ? err.message : 'Failed to trigger shuffle');
        } finally {
            setShuffling(false);
        }
    };

    if (loading && !data) {
        return (
            <Layout title="Dashboard">
                <div className="flex-1 flex items-center justify-center min-h-[400px]">
                    <motion.div
                        animate={{ rotate: 360 }}
                        transition={{ duration: 1, repeat: Infinity, ease: "linear" }}
                        className="w-10 h-10 border-4 border-primary border-t-transparent rounded-full shadow-lg shadow-primary/20"
                    />
                </div>
            </Layout>
        );
    }

    return (
        <Layout title="Overview">
            <div className="space-y-8">
                {error && (
                    <div className="p-4 rounded-2xl bg-destructive/10 border border-destructive/20 text-destructive text-sm flex items-center gap-3">
                        <BarChart3 className="w-5 h-5 rotate-180" />
                        {error}
                    </div>
                )}

                {/* Hero Section */}
                <motion.div
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    className="grid lg:grid-cols-3 gap-8"
                >
                    <div className="lg:col-span-2 space-y-8">
                        <div className="glass p-10 rounded-[2.5rem] relative overflow-hidden group">
                            <div className="absolute top-0 right-0 p-8">
                                <BarChart3 className="w-32 h-32 text-primary/5 -rotate-12 group-hover:rotate-0 transition-transform duration-700" />
                            </div>
                            <div className="relative">
                                <span className="px-4 py-1.5 rounded-full bg-primary/10 text-primary text-xs font-bold uppercase tracking-wider mb-6 inline-block">
                                    Active Schedule
                                </span>
                                <h2 className="text-4xl font-extrabold mb-4 tracking-tight leading-tight">
                                    Next Assignment Shuffling in <br />
                                    <span className="text-primary italic">{data?.next_shuffle_in_days ?? 0} Days</span>
                                </h2>
                                <p className="text-muted-foreground max-w-md mb-8 leading-relaxed">
                                    The system is currently on track. No manual intervention is required at this time.
                                </p>
                                <button className="px-8 py-4 rounded-2xl bg-foreground text-background font-bold hover:scale-[1.02] active:scale-[0.98] transition-all duration-300 shadow-xl shadow-foreground/10">
                                    View Schedule History
                                </button>
                            </div>
                        </div>

                        {/* Quick Actions */}
                        <div className="grid sm:grid-cols-2 gap-6">
                            <div className="glass p-8 rounded-3xl hover:border-primary/30 transition-colors group cursor-pointer border border-border/30">
                                <div className="w-12 h-12 rounded-2xl bg-secondary/10 flex items-center justify-center mb-4 group-hover:scale-110 transition-transform duration-300">
                                    <Users className="text-secondary w-6 h-6" />
                                </div>
                                <h3 className="font-bold mb-1">Manage Groups</h3>
                                <p className="text-sm text-muted-foreground">{data?.people.length ?? 0} active members synced</p>
                            </div>
                            <div
                                onClick={triggerShuffle}
                                className={`glass p-8 rounded-3xl hover:border-accent/30 transition-colors group cursor-pointer border border-border/30 ${shuffling ? 'opacity-50 pointer-events-none' : ''}`}
                            >
                                <div className="w-12 h-12 rounded-2xl bg-accent/10 flex items-center justify-center mb-4 group-hover:scale-110 transition-transform duration-300">
                                    <Calendar className={`text-accent w-6 h-6 ${shuffling ? 'animate-bounce' : ''}`} />
                                </div>
                                <h3 className="font-bold mb-1">Manual Trigger</h3>
                                <p className="text-sm text-muted-foreground">Force re-shuffling now</p>
                            </div>
                        </div>
                    </div>

                    {/* Info Cards */}
                    <div className="space-y-6">
                        <div className="glass p-8 rounded-3xl border border-border/30 shadow-2xl shadow-black/5">
                            <h3 className="font-bold mb-6 flex items-center gap-2">
                                Recent Activity
                            </h3>
                            <div className="space-y-6">
                                {data?.recent_assignments.map((assignment) => (
                                    <div key={assignment.id} className="flex gap-4 items-start group">
                                        <div className="w-2.5 h-2.5 rounded-full bg-primary mt-1.5 group-hover:scale-150 transition-transform duration-300 shadow-[0_0_10px_rgba(var(--primary),0.5)]" />
                                        <div className="flex-1 border-b border-border/30 pb-4 group-last:border-0">
                                            <p className="text-sm font-bold mb-0.5">Assigned to {assignment.task_name}</p>
                                            <p className="text-xs text-muted-foreground">
                                                {new Date(assignment.assigned_at).toLocaleDateString()}
                                            </p>
                                        </div>
                                    </div>
                                ))}
                                {(!data || data.recent_assignments.length === 0) && (
                                    <p className="text-xs text-muted-foreground italic text-center py-4">No recent assignments found</p>
                                )}
                            </div>
                            <button className="w-full mt-6 py-3 rounded-xl hover:bg-muted/50 transition-colors text-sm font-semibold border border-border/30">
                                View Audit Log
                            </button>
                        </div>

                        <div className="p-8 rounded-3xl bg-foreground text-background relative overflow-hidden group">
                            <div className="absolute top-0 right-0 w-32 h-32 bg-primary/20 rounded-full blur-3xl group-hover:scale-150 transition-transform duration-1000" />
                            <h3 className="font-bold mb-2 relative z-10 text-lg">Pro Support</h3>
                            <p className="text-sm text-background/60 mb-6 relative z-10 leading-relaxed">
                                Advanced analytics and cloud backup for your workspace.
                            </p>
                            <button className="w-full py-3 rounded-xl bg-background text-foreground font-bold text-sm hover:translate-y-[-2px] transition-transform relative z-10">
                                Upgrade Now
                            </button>
                        </div>
                    </div>
                </motion.div>
            </div>
        </Layout>
    );
};
