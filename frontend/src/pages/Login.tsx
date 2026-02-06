import { motion } from "framer-motion";
import { ShieldCheck, Mail, Lock, ArrowRight, Eye, EyeOff } from "lucide-react";
import { useState } from "react";
import { useAuth } from "../context/AuthContext";

export const Login = () => {
    const { signIn, signUp } = useAuth();
    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");
    const [showPassword, setShowPassword] = useState(false);
    const [isLogin, setIsLogin] = useState(true);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [success, setSuccess] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setLoading(true);
        setError(null);
        setSuccess(null);
        try {
            if (isLogin) {
                await signIn(email, password);
            } else {
                await signUp(email, password);
                setSuccess("Check your email for confirmation!");
            }
        } catch (err) {
            setError(err instanceof Error ? err.message : "Authentication failed");
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="min-h-screen bg-background flex items-center justify-center p-6 relative overflow-hidden">
            {/* Dynamic Background */}
            <div className="absolute inset-0">
                <div className="absolute top-[-20%] left-[-10%] w-[60%] h-[60%] bg-primary/20 rounded-full blur-[150px] animate-pulse" />
                <div className="absolute bottom-[-20%] right-[-10%] w-[60%] h-[60%] bg-accent/20 rounded-full blur-[150px] animate-pulse delay-700" />
            </div>

            <motion.div
                initial={{ opacity: 0, scale: 0.95 }}
                animate={{ opacity: 1, scale: 1 }}
                className="w-full max-w-lg relative z-10"
            >
                <div className="glass p-12 rounded-[3rem] border border-white/10 shadow-2xl backdrop-blur-3xl bg-white/5">
                    <div className="text-center mb-12">
                        <motion.div
                            initial={{ rotate: -10, scale: 0.8 }}
                            animate={{ rotate: 0, scale: 1 }}
                            transition={{ type: "spring", damping: 10 }}
                            className="w-20 h-20 rounded-3xl bg-gradient-to-br from-primary to-accent flex items-center justify-center shadow-2xl shadow-primary/40 mx-auto mb-8"
                        >
                            <ShieldCheck className="text-white w-10 h-10" />
                        </motion.div>
                        <h1 className="text-4xl font-black tracking-tight mb-2 bg-clip-text text-transparent bg-gradient-to-r from-foreground to-foreground/80">
                            VividShift
                        </h1>
                        <p className="text-muted-foreground font-medium">
                            {isLogin ? "Secure Admin Control Panel" : "Create your account"}
                        </p>
                    </div>

                    <form onSubmit={handleSubmit} className="space-y-6">
                        <div className="space-y-2">
                            <div className="relative group">
                                <Mail className="absolute left-6 top-1/2 -translate-y-1/2 text-muted-foreground w-5 h-5 group-focus-within:text-primary transition-colors" />
                                <input
                                    required
                                    type="email"
                                    value={email}
                                    onChange={(e) => setEmail(e.target.value)}
                                    placeholder="admin@vividshift.io"
                                    className="w-full bg-muted/30 border border-border/50 rounded-2xl pl-14 pr-6 py-5 focus:outline-none focus:ring-2 focus:ring-primary/20 transition-all font-medium text-lg placeholder:text-muted-foreground/30"
                                />
                            </div>
                        </div>

                        <div className="space-y-2">
                            <div className="relative group">
                                <Lock className="absolute left-6 top-1/2 -translate-y-1/2 text-muted-foreground w-5 h-5 group-focus-within:text-primary transition-colors" />
                                <input
                                    required
                                    type={showPassword ? "text" : "password"}
                                    value={password}
                                    onChange={(e) => setPassword(e.target.value)}
                                    placeholder="••••••••"
                                    className="w-full bg-muted/30 border border-border/50 rounded-2xl pl-14 pr-16 py-5 focus:outline-none focus:ring-2 focus:ring-primary/20 transition-all font-medium text-lg placeholder:text-muted-foreground/30"
                                />
                                <button
                                    type="button"
                                    onClick={() => setShowPassword(!showPassword)}
                                    className="absolute right-6 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-primary transition-colors"
                                >
                                    {showPassword ? <EyeOff className="w-5 h-5" /> : <Eye className="w-5 h-5" />}
                                </button>
                            </div>
                        </div>

                        {error && (
                            <motion.div
                                initial={{ opacity: 0, y: -10 }}
                                animate={{ opacity: 1, y: 0 }}
                                className="p-4 rounded-xl bg-destructive/10 border border-destructive/20 text-destructive text-sm font-medium text-center"
                            >
                                {error}
                            </motion.div>
                        )}

                        {success && (
                            <motion.div
                                initial={{ opacity: 0, y: -10 }}
                                animate={{ opacity: 1, y: 0 }}
                                className="p-4 rounded-xl bg-green-500/10 border border-green-500/20 text-green-500 text-sm font-medium text-center"
                            >
                                {success}
                            </motion.div>
                        )}

                        <button
                            disabled={loading}
                            type="submit"
                            className="w-full h-20 rounded-2xl bg-foreground text-background font-black text-xl hover:scale-[1.02] active:scale-[0.98] transition-all shadow-2xl shadow-foreground/20 flex items-center justify-center gap-4 group"
                        >
                            {loading ? (
                                <div className="w-8 h-8 border-4 border-background border-t-transparent rounded-full animate-spin" />
                            ) : (
                                <>
                                    {isLogin ? "Enter Dashboard" : "Sign Up"}
                                    <ArrowRight className="w-6 h-6 group-hover:translate-x-2 transition-transform" />
                                </>
                            )}
                        </button>

                        <button
                            type="button"
                            onClick={() => setIsLogin(!isLogin)}
                            className="w-full text-center text-sm font-semibold text-primary hover:text-primary/80 transition-colors"
                        >
                            {isLogin ? "Need an account? Sign Up" : "Already have an account? Login"}
                        </button>
                    </form>

                    <p className="mt-12 text-center text-xs text-muted-foreground/40 font-medium tracking-widest uppercase">
                        &copy; 2026 VividShift Enterprise. All rights reserved.
                    </p>
                </div>
            </motion.div>
        </div>
    );
};
