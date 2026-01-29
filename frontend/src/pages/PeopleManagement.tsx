import { motion } from "framer-motion";
import {
    Users,
    Plus,
    Trash2,
    CheckCircle,
    XCircle,
} from "lucide-react";
import { useState, useEffect } from "react";
import { useAuth } from "../context/AuthContext";
import { Layout } from "../components/layout/Layout";
import type { Person } from "../types";

export const PeopleManagement = () => {
    const { session } = useAuth();
    const [people, setPeople] = useState<Person[]>([]);
    const [loading, setLoading] = useState(true);

    const [isAdding, setIsAdding] = useState(false);
    const [newPerson, setNewPerson] = useState({ name: "", group_type: "A", active: true });

    const fetchPeople = async () => {
        try {
            setLoading(true);
            const resp = await fetch('/api/people', {
                headers: {
                    'Authorization': `Bearer ${session?.access_token}`,
                }
            });
            if (!resp.ok) throw new Error('Failed to fetch people');
            const json = await resp.json();
            setPeople(json);
        } catch (err) {
            console.error(err);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        if (session) fetchPeople();
    }, [session]);

    const handleAdd = async (e: React.FormEvent) => {
        e.preventDefault();
        try {
            const resp = await fetch('/api/people', {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${session?.access_token}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(newPerson),
            });
            if (!resp.ok) throw new Error('Failed to add person');
            setIsAdding(false);
            setNewPerson({ name: "", group_type: "A", active: true });
            fetchPeople();
        } catch (err) {
            alert(err instanceof Error ? err.message : 'Failed to add person');
        }
    };

    const handleToggleActive = async (person: Person) => {
        try {
            const resp = await fetch(`/api/people/${person.id}`, {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${session?.access_token}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ active: !person.active }),
            });
            if (!resp.ok) throw new Error('Failed to update person');
            fetchPeople();
        } catch (err) {
            alert(err instanceof Error ? err.message : 'Failed to update person');
        }
    };

    const handleDelete = async (id: number) => {
        if (!window.confirm("Are you sure? This will permanently remove the person.")) return;
        try {
            const resp = await fetch(`/api/people/${id}`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${session?.access_token}`,
                },
            });
            if (!resp.ok) throw new Error('Failed to delete person');
            fetchPeople();
        } catch (err) {
            alert(err instanceof Error ? err.message : 'Failed to delete person');
        }
    };

    return (
        <Layout title="People Management">
            <div className="space-y-6">
                <div className="flex justify-between items-center bg-muted/30 p-4 rounded-2xl border border-border/50">
                    <div>
                        <h2 className="text-lg font-bold">Manage Members</h2>
                        <p className="text-sm text-muted-foreground">Add or remove people from the rotation</p>
                    </div>
                    <button
                        onClick={() => setIsAdding(!isAdding)}
                        className="flex items-center gap-2 px-6 py-2.5 rounded-xl bg-primary text-white font-bold hover:scale-[1.02] active:scale-[0.98] transition-all"
                    >
                        {isAdding ? "Cancel" : <><Plus className="w-4 h-4" /> Add Person</>}
                    </button>
                </div>

                {isAdding && (
                    <motion.div
                        initial={{ opacity: 0, scale: 0.95 }}
                        animate={{ opacity: 1, scale: 1 }}
                        className="glass p-6 rounded-2xl border border-primary/20 bg-primary/5"
                    >
                        <form onSubmit={handleAdd} className="flex flex-wrap gap-4 items-end">
                            <div className="flex-1 min-w-[200px] space-y-1.5">
                                <label className="text-xs font-bold uppercase tracking-wider text-muted-foreground ml-1">Full Name</label>
                                <input
                                    autoFocus
                                    required
                                    type="text"
                                    value={newPerson.name}
                                    onChange={(e) => setNewPerson({ ...newPerson, name: e.target.value })}
                                    className="w-full bg-background border border-border rounded-xl px-4 py-2 focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none"
                                    placeholder="e.g. John Doe"
                                />
                            </div>
                            <div className="w-32 space-y-1.5">
                                <label className="text-xs font-bold uppercase tracking-wider text-muted-foreground ml-1">Group</label>
                                <select
                                    value={newPerson.group_type}
                                    onChange={(e) => setNewPerson({ ...newPerson, group_type: e.target.value })}
                                    className="w-full bg-background border border-border rounded-xl px-4 py-2 focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none"
                                >
                                    <option value="A">Group A</option>
                                    <option value="B">Group B</option>
                                </select>
                            </div>
                            <button
                                type="submit"
                                className="px-8 py-2 rounded-xl bg-foreground text-background font-bold h-[42px]"
                            >
                                Save
                            </button>
                        </form>
                    </motion.div>
                )}

                <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                    {people.map((person) => (
                        <motion.div
                            layout
                            key={person.id}
                            className={`glass p-6 rounded-2xl border transition-all ${person.active ? 'border-border/30 hover:border-primary/50' : 'opacity-60 border-dashed border-border'}`}
                        >
                            <div className="flex justify-between items-start mb-4">
                                <div className="w-12 h-12 rounded-xl bg-muted overflow-hidden">
                                    <img src={`https://api.dicebear.com/7.x/avataaars/svg?seed=${person.name}`} alt={person.name} />
                                </div>
                                <div className="flex gap-1">
                                    <button
                                        onClick={() => handleToggleActive(person)}
                                        className={`p-2 rounded-lg transition-colors ${person.active ? 'hover:bg-warning/10 text-warning' : 'hover:bg-success/10 text-success'}`}
                                        title={person.active ? "Set Inactive" : "Set Active"}
                                    >
                                        {person.active ? <XCircle className="w-5 h-5" /> : <CheckCircle className="w-5 h-5" />}
                                    </button>
                                    <button
                                        onClick={() => handleDelete(person.id)}
                                        className="p-2 rounded-lg hover:bg-destructive/10 text-destructive transition-colors"
                                    >
                                        <Trash2 className="w-5 h-5" />
                                    </button>
                                </div>
                            </div>
                            <div className="space-y-1">
                                <h3 className="font-bold text-lg leading-none">{person.name}</h3>
                                <div className="flex items-center gap-2">
                                    <span className={`text-[10px] font-bold uppercase tracking-widest px-2 py-0.5 rounded-full ${person.group_type === 'A' ? 'bg-primary/10 text-primary' : 'bg-secondary/10 text-secondary'}`}>
                                        Group {person.group_type}
                                    </span>
                                    <span className={`text-[10px] font-bold uppercase tracking-widest px-2 py-0.5 rounded-full ${person.active ? 'bg-success/10 text-success' : 'bg-muted text-muted-foreground'}`}>
                                        {person.active ? 'Active' : 'Inactive'}
                                    </span>
                                </div>
                            </div>
                        </motion.div>
                    ))}

                    {people.length === 0 && !loading && (
                        <div className="col-span-full py-20 text-center glass rounded-3xl border border-dashed border-border/50">
                            <Users className="w-12 h-12 text-muted-foreground mx-auto mb-4 opacity-20" />
                            <p className="text-muted-foreground">No people found. Add someone to get started!</p>
                        </div>
                    )}
                </div>
            </div>
        </Layout>
    );
};
