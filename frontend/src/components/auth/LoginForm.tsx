import { useState } from 'react';
import { supabase } from '@/lib/supabase';
import { motion } from 'framer-motion';
import { Lock, Mail, Loader2 } from 'lucide-react';

export const LoginForm = () => {
    const [loading, setLoading] = useState(false);
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [error, setError] = useState<string | null>(null);

    const handleLogin = async (e: React.FormEvent) => {
        e.preventDefault();
        setLoading(true);
        setError(null);

        const { error } = await supabase.auth.signInWithPassword({
            email,
            password,
        });

        if (error) {
            setError(error.message);
        }
        setLoading(false);
    };

    return (
        <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            className="w-full max-w-md p-8 glass rounded-[2rem] border border-border/30 shadow-2xl"
        >
            <div className="text-center mb-8">
                <h2 className="text-3xl font-bold tracking-tight mb-2">Welcome Back</h2>
                <p className="text-muted-foreground text-sm">Enter your credentials to access the dashboard</p>
            </div>

            <form onSubmit={handleLogin} className="space-y-6">
                <div className="space-y-2">
                    <label className="text-sm font-medium px-1">Email address</label>
                    <div className="relative group">
                        <Mail className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-muted-foreground transition-colors group-focus-within:text-primary" />
                        <input
                            type="email"
                            required
                            value={email}
                            onChange={(e) => setEmail(e.target.value)}
                            placeholder="name@example.com"
                            className="w-full pl-12 pr-4 py-4 rounded-2xl bg-muted/50 border border-border/50 focus:border-primary focus:ring-4 focus:ring-primary/10 transition-all outline-none"
                        />
                    </div>
                </div>

                <div className="space-y-2">
                    <label className="text-sm font-medium px-1">Password</label>
                    <div className="relative group">
                        <Lock className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-muted-foreground transition-colors group-focus-within:text-primary" />
                        <input
                            type="password"
                            required
                            value={password}
                            onChange={(e) => setPassword(e.target.value)}
                            placeholder="••••••••"
                            className="w-full pl-12 pr-4 py-4 rounded-2xl bg-muted/50 border border-border/50 focus:border-primary focus:ring-4 focus:ring-primary/10 transition-all outline-none"
                        />
                    </div>
                </div>

                {error && (
                    <motion.p
                        initial={{ opacity: 0, scale: 0.95 }}
                        animate={{ opacity: 1, scale: 1 }}
                        className="text-destructive text-sm bg-destructive/10 px-4 py-3 rounded-xl border border-destructive/20 text-center font-medium"
                    >
                        {error}
                    </motion.p>
                )}

                <button
                    disabled={loading}
                    className="w-full py-4 rounded-2xl bg-foreground text-background font-bold hover:scale-[1.02] active:scale-[0.98] transition-all duration-300 shadow-xl shadow-foreground/10 disabled:opacity-50 disabled:scale-100 flex items-center justify-center gap-2"
                >
                    {loading ? <Loader2 className="w-5 h-5 animate-spin" /> : "Sign In"}
                </button>
            </form>

            <div className="mt-8 text-center">
                <p className="text-sm text-muted-foreground">
                    Don't have an account? <button className="text-primary font-bold hover:underline">Contact Admin</button>
                </p>
            </div>
        </motion.div>
    );
};
