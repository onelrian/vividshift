import { motion } from "framer-motion";
import {
    Save,
    RefreshCw,
    UserCircle,
    KeyRound,
    Eye,
    EyeOff,
} from "lucide-react";
import { useState, useEffect } from "react";
import { useAuth } from "../context/AuthContext";
import { Layout } from "../components/layout/Layout";
import type { Setting } from "../types";
import { supabase } from "../lib/supabase";

export const Settings = () => {
    const { session, isAdmin } = useAuth();
    const [settings, setSettings] = useState<Setting[]>([]);
    const [loading, setLoading] = useState(true);
    const [saving, setSaving] = useState(false);
    const [profile, setProfile] = useState<{ id: string, username: string, email: string } | null>(null);
    const [newUsername, setNewUsername] = useState("");
    const [password, setPassword] = useState("");
    const [confirmPassword, setConfirmPassword] = useState("");
    const [updatingProfile, setUpdatingProfile] = useState(false);
    const [showPassword, setShowPassword] = useState(false);
    const [updatingPassword, setUpdatingPassword] = useState(false);

    const fetchProfile = async () => {
        try {
            const resp = await fetch('/api/auth/profile', {
                headers: {
                    'Authorization': `Bearer ${session?.access_token}`,
                }
            });
            if (resp.ok) {
                const data = await resp.json();
                setProfile(data);
                setNewUsername(data.username);
            }
        } catch (err) {
            console.error("Failed to fetch profile", err);
        }
    };

    const fetchSettings = async () => {
        try {
            setLoading(true);
            const resp = await fetch('/api/settings', {
                headers: {
                    'Authorization': `Bearer ${session?.access_token}`,
                }
            });
            if (!resp.ok) throw new Error('Failed to fetch settings');
            const json = await resp.json();
            setSettings(json);
        } catch (err) {
            console.error(err);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        if (session) {
            fetchSettings();
            fetchProfile();
        }
    }, [session]);

    const handleUpdateProfile = async (e: React.FormEvent) => {
        e.preventDefault();
        try {
            setUpdatingProfile(true);
            const resp = await fetch('/api/auth/profile', {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${session?.access_token}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ username: newUsername }),
            });
            if (!resp.ok) throw new Error('Failed to update profile');
            alert("Username updated successfully!");
            fetchProfile();
        } catch (err) {
            alert(err instanceof Error ? err.message : 'Failed to update profile');
        } finally {
            setUpdatingProfile(false);
        }
    };

    const handleUpdatePassword = async (e: React.FormEvent) => {
        e.preventDefault();
        if (password !== confirmPassword) {
            alert("Passwords do not match");
            return;
        }
        try {
            setUpdatingPassword(true);
            const { error } = await supabase.auth.updateUser({ password });
            if (error) throw error;
            alert("Password updated successfully!");
            setPassword("");
            setConfirmPassword("");
        } catch (err) {
            alert(err instanceof Error ? err.message : 'Failed to update password');
        } finally {
            setUpdatingPassword(false);
        }
    };

    const updateSetting = (key: string, value: string) => {
        setSettings(prev => prev.map(s => s.key === key ? { ...s, value } : s));
    };

    const handleSave = async () => {
        try {
            setSaving(true);
            const resp = await fetch('/api/settings', {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${session?.access_token}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(settings),
            });
            if (!resp.ok) throw new Error('Failed to save settings');
            alert("Settings saved successfully!");
            fetchSettings();
        } catch (err) {
            alert(err instanceof Error ? err.message : 'Failed to save settings');
        } finally {
            setSaving(false);
        }
    };

    return (
        <Layout title="Account & System Settings">
            <div className="max-w-3xl space-y-8">
                {/* Profile Section */}
                <div className="glass p-10 rounded-[2.5rem] border border-border/30">
                    <div className="flex items-center gap-4 mb-8">
                        <div className="w-12 h-12 rounded-2xl bg-primary/10 flex items-center justify-center">
                            <UserCircle className="text-primary w-6 h-6" />
                        </div>
                        <div>
                            <h2 className="text-2xl font-bold mb-1">Your Profile</h2>
                            <p className="text-sm text-muted-foreground">{profile?.email}</p>
                        </div>
                    </div>

                    <form onSubmit={handleUpdateProfile} className="space-y-6">
                        <div className="space-y-3">
                            <label className="text-sm font-bold uppercase tracking-widest text-muted-foreground ml-1">
                                Username
                            </label>
                            <input
                                type="text"
                                value={newUsername}
                                onChange={(e) => setNewUsername(e.target.value)}
                                className="w-full bg-muted/30 border border-border/50 rounded-2xl px-6 py-4 focus:outline-none focus:ring-2 focus:ring-primary/20 transition-all font-medium"
                                placeholder="Enter username"
                                required
                            />
                        </div>
                        <button
                            type="submit"
                            disabled={updatingProfile || newUsername === profile?.username}
                            className="flex items-center gap-2 px-6 py-3 rounded-xl bg-primary text-primary-foreground font-bold hover:scale-[1.02] active:scale-[0.98] transition-all disabled:opacity-50"
                        >
                            <Save className="w-4 h-4" />
                            {updatingProfile ? "Updating..." : "Update Username"}
                        </button>
                    </form>

                    <div className="my-10 border-t border-border/30" />

                    <div className="flex items-center gap-4 mb-8">
                        <div className="w-12 h-12 rounded-2xl bg-accent/10 flex items-center justify-center">
                            <KeyRound className="text-accent w-6 h-6" />
                        </div>
                        <div>
                            <h2 className="text-2xl font-bold mb-1">Security</h2>
                            <p className="text-sm text-muted-foreground">Manage your account password</p>
                        </div>
                    </div>

                    <form onSubmit={handleUpdatePassword} className="space-y-6">
                        <div className="grid sm:grid-cols-2 gap-6">
                            <div className="space-y-3">
                                <label className="text-sm font-bold uppercase tracking-widest text-muted-foreground ml-1">
                                    New Password
                                </label>
                                <div className="relative group">
                                    <input
                                        type={showPassword ? "text" : "password"}
                                        value={password}
                                        onChange={(e) => setPassword(e.target.value)}
                                        className="w-full bg-muted/30 border border-border/50 rounded-2xl px-6 pr-14 py-4 focus:outline-none focus:ring-2 focus:ring-primary/20 transition-all font-medium"
                                        placeholder="••••••••"
                                        required
                                    />
                                    <button
                                        type="button"
                                        onClick={() => setShowPassword(!showPassword)}
                                        className="absolute right-5 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-primary transition-colors"
                                    >
                                        {showPassword ? <EyeOff className="w-5 h-5" /> : <Eye className="w-5 h-5" />}
                                    </button>
                                </div>
                            </div>
                            <div className="space-y-3">
                                <label className="text-sm font-bold uppercase tracking-widest text-muted-foreground ml-1">
                                    Confirm Password
                                </label>
                                <div className="relative group">
                                    <input
                                        type={showPassword ? "text" : "password"}
                                        value={confirmPassword}
                                        onChange={(e) => setConfirmPassword(e.target.value)}
                                        className="w-full bg-muted/30 border border-border/50 rounded-2xl px-6 pr-14 py-4 focus:outline-none focus:ring-2 focus:ring-primary/20 transition-all font-medium"
                                        placeholder="••••••••"
                                        required
                                    />
                                </div>
                            </div>
                        </div>
                        <button
                            type="submit"
                            disabled={updatingPassword || !password}
                            className="flex items-center gap-2 px-6 py-3 rounded-xl bg-foreground text-background font-bold hover:scale-[1.02] active:scale-[0.98] transition-all disabled:opacity-50"
                        >
                            <KeyRound className="w-4 h-4" />
                            {updatingPassword ? "Updating..." : "Change Password"}
                        </button>
                    </form>
                </div>

                {/* System Configuration - Admin Only */}
                {isAdmin && (
                    <>
                        <div className="glass p-10 rounded-[2.5rem] border border-border/30">
                            <div className="flex items-center justify-between mb-8">
                                <div>
                                    <h2 className="text-2xl font-bold mb-1">Configuration</h2>
                                    <p className="text-sm text-muted-foreground">Adjust system parameters and integration settings</p>
                                </div>
                                <button
                                    onClick={fetchSettings}
                                    className="p-3 rounded-xl hover:bg-muted/50 border border-border/30 transition-colors"
                                    title="Refresh Settings"
                                >
                                    <RefreshCw className={`w-5 h-5 ${loading ? 'animate-spin' : ''}`} />
                                </button>
                            </div>

                            <div className="space-y-8">
                                {settings.map((setting) => (
                                    <div key={setting.key} className="space-y-3">
                                        <div className="flex justify-between items-end">
                                            <label className="text-sm font-bold uppercase tracking-widest text-muted-foreground ml-1">
                                                {setting.key.replace(/_/g, ' ')}
                                            </label>
                                        </div>
                                        {setting.value === 'true' || setting.value === 'false' ? (
                                            <div className="flex items-center gap-4">
                                                <button
                                                    onClick={() => updateSetting(setting.key, setting.value === 'true' ? 'false' : 'true')}
                                                    className={`w-14 h-8 rounded-full transition-colors relative ${setting.value === 'true' ? 'bg-primary' : 'bg-muted'}`}
                                                >
                                                    <motion.div
                                                        animate={{ x: setting.value === 'true' ? 24 : 4 }}
                                                        className="w-6 h-6 rounded-full bg-white absolute top-1 shadow-sm"
                                                    />
                                                </button>
                                                <span className="text-sm font-medium">{setting.value === 'true' ? 'Enabled' : 'Disabled'}</span>
                                            </div>
                                        ) : (
                                            <input
                                                type="text"
                                                value={setting.value}
                                                onChange={(e) => updateSetting(setting.key, e.target.value)}
                                                className="w-full bg-muted/30 border border-border/50 rounded-2xl px-6 py-4 focus:outline-none focus:ring-2 focus:ring-primary/20 transition-all font-medium"
                                            />
                                        )}
                                    </div>
                                ))}

                                {settings.length === 0 && !loading && (
                                    <p className="text-center text-muted-foreground italic py-10 opacity-50">No configurable settings found in database.</p>
                                )}

                                <div className="pt-4 border-t border-border/30">
                                    <button
                                        disabled={saving || loading || !isAdmin}
                                        onClick={handleSave}
                                        className="w-full flex items-center justify-center gap-3 px-8 py-4 rounded-2xl bg-foreground text-background font-bold hover:scale-[1.01] active:scale-[0.99] transition-all shadow-xl shadow-foreground/10 disabled:opacity-50"
                                    >
                                        <Save className="w-5 h-5" />
                                        {saving ? "Saving Changes..." : "Save Configuration"}
                                    </button>
                                </div>
                            </div>
                        </div>

                        <div className="glass p-8 rounded-3xl border border-destructive/20 bg-destructive/5">
                            <h3 className="text-destructive font-bold mb-2">Danger Zone</h3>
                            <p className="text-sm text-muted-foreground mb-6">Irreversible actions that affect the entire workspace data.</p>
                            <button className="px-6 py-3 rounded-xl border border-destructive/30 text-destructive font-bold text-sm hover:bg-destructive/10 transition-colors">
                                Reset Assignment History
                            </button>
                        </div>
                    </>
                )}
            </div>
        </Layout>
    );
};
