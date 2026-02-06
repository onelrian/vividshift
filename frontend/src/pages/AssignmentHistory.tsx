import { motion } from "framer-motion";
import {
    Calendar,
    Search,
    Filter,
    Download,
} from "lucide-react";
import { useState, useEffect } from "react";
import { useAuth } from "../context/AuthContext";
import { Layout } from "../components/layout/Layout";
import type { Assignment } from "../types";

export const AssignmentHistory = () => {
    const { session } = useAuth();
    const [assignments, setAssignments] = useState<Assignment[]>([]);
    const [loading, setLoading] = useState(true);
    const [searchTerm, setSearchTerm] = useState("");

    const fetchHistory = async () => {
        try {
            setLoading(true);
            const resp = await fetch('/api/assignments', {
                headers: {
                    'Authorization': `Bearer ${session?.access_token}`,
                }
            });
            if (!resp.ok) throw new Error('Failed to fetch history');
            const json = await resp.json();
            setAssignments(json);
        } catch (err) {
            console.error(err);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        if (session) fetchHistory();
    }, [session]);

    const filtered = assignments.filter(a =>
        a.task_name.toLowerCase().includes(searchTerm.toLowerCase())
    );

    return (
        <Layout title="Assignment History">
            <div className="space-y-6">
                <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                    <div className="relative flex-1 max-w-md">
                        <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-muted-foreground w-4 h-4" />
                        <input
                            type="text"
                            placeholder="Search assignments..."
                            value={searchTerm}
                            onChange={(e) => setSearchTerm(e.target.value)}
                            className="w-full bg-muted/30 border border-border/50 rounded-2xl pl-12 pr-4 py-3 focus:outline-none focus:ring-2 focus:ring-primary/20 transition-all"
                        />
                    </div>
                    <div className="flex gap-3">
                        <button className="flex items-center gap-2 px-4 py-2 rounded-xl bg-muted/40 border border-border/50 hover:bg-muted/60 transition-colors text-sm font-medium">
                            <Filter className="w-4 h-4" /> Filter
                        </button>
                        <button className="flex items-center gap-2 px-4 py-2 rounded-xl bg-muted/40 border border-border/50 hover:bg-muted/60 transition-colors text-sm font-medium">
                            <Download className="w-4 h-4" /> Export
                        </button>
                    </div>
                </div>

                <div className="glass rounded-[2rem] border border-border/30 overflow-hidden">
                    <table className="w-full text-left border-collapse">
                        <thead>
                            <tr className="bg-muted/30 border-b border-border/50">
                                <th className="px-8 py-5 text-[10px] font-bold uppercase tracking-widest text-muted-foreground">ID</th>
                                <th className="px-8 py-5 text-[10px] font-bold uppercase tracking-widest text-muted-foreground">Task / Role</th>
                                <th className="px-8 py-5 text-[10px] font-bold uppercase tracking-widest text-muted-foreground">Date Assigned</th>
                                <th className="px-8 py-5 text-[10px] font-bold uppercase tracking-widest text-muted-foreground">Status</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-border/30">
                            {filtered.map((item, i) => (
                                <motion.tr
                                    initial={{ opacity: 0, x: -10 }}
                                    animate={{ opacity: 1, x: 0 }}
                                    transition={{ delay: i * 0.03, duration: 0.2 }}
                                    key={item.id}
                                    className="hover:bg-primary/5 transition-colors group"
                                >
                                    <td className="px-8 py-5 text-sm font-mono text-muted-foreground">#SHF-{item.id}</td>
                                    <td className="px-8 py-5">
                                        <div className="flex items-center gap-3">
                                            <div className="w-8 h-8 rounded-lg bg-accent/10 flex items-center justify-center">
                                                <Calendar className="w-4 h-4 text-accent" />
                                            </div>
                                            <span className="font-semibold group-hover:text-primary transition-colors">{item.task_name}</span>
                                        </div>
                                    </td>
                                    <td className="px-8 py-5 text-sm text-muted-foreground">
                                        {new Date(item.assigned_at).toLocaleString()}
                                    </td>
                                    <td className="px-8 py-5">
                                        <span className="px-3 py-1 rounded-full bg-success/10 text-success text-[10px] font-bold uppercase tracking-wider">
                                            Completed
                                        </span>
                                    </td>
                                </motion.tr>
                            ))}
                            {filtered.length === 0 && !loading && (
                                <tr>
                                    <td colSpan={4} className="px-8 py-10 text-center text-muted-foreground italic">
                                        No matching assignment history found tokens.
                                    </td>
                                </tr>
                            )}
                        </tbody>
                    </table>
                </div>
            </div>
        </Layout>
    );
};
