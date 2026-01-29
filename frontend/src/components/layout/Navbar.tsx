import { motion } from "framer-motion";
import {
    BarChart3,
    Calendar,
    Settings as SettingsIcon,
    Users,
    UserCog,
    ShieldCheck,
    ChevronRight,
    LogOut
} from "lucide-react";
import { Link, useLocation } from "react-router-dom";
import { useAuth } from "../../context/AuthContext";

export const Navbar = () => {
    const { isAdmin, signOut } = useAuth();
    const location = useLocation();

    const navItems = [
        { icon: BarChart3, label: "Dashboard", href: "/" },
        { icon: Calendar, label: "History", href: "/history" },
        ...(isAdmin ? [
            { icon: Users, label: "People", href: "/people" },
            { icon: UserCog, label: "User Management", href: "/admin/users" }
        ] : []),
        { icon: SettingsIcon, label: "Settings", href: "/settings" },
    ];

    return (
        <aside className="w-72 flex-shrink-0 glass border-r border-border m-4 rounded-3xl flex flex-col hidden lg:flex">
            <div className="p-8">
                <motion.div
                    initial={{ opacity: 0, x: -20 }}
                    animate={{ opacity: 1, x: 0 }}
                    className="flex items-center gap-3"
                >
                    <Link to="/" className="flex items-center gap-3">
                        <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-primary to-accent flex items-center justify-center shadow-lg shadow-primary/20">
                            <ShieldCheck className="text-white w-6 h-6" />
                        </div>
                        <span className="text-2xl font-bold tracking-tight bg-clip-text text-transparent bg-gradient-to-r from-foreground to-foreground/70">
                            VividShift
                        </span>
                    </Link>
                </motion.div>
            </div>

            <nav className="flex-1 px-4 space-y-2 mt-4">
                {navItems.map((item, i) => {
                    const isActive = location.pathname === item.href;
                    return (
                        <Link key={item.label} to={item.href}>
                            <motion.div
                                initial={{ opacity: 0, x: -20 }}
                                animate={{ opacity: 1, x: 0 }}
                                transition={{ delay: i * 0.1 }}
                                className={`w-full flex items-center gap-4 px-6 py-4 rounded-2xl transition-all duration-300 group ${isActive
                                    ? "bg-primary/10 text-primary shadow-sm"
                                    : "hover:bg-muted/50 text-muted-foreground hover:text-foreground"
                                    }`}
                            >
                                <item.icon className={`w-5 h-5 transition-transform duration-300 ${isActive ? "scale-110" : "group-hover:scale-110"}`} />
                                <span className="font-medium">{item.label}</span>
                                {isActive && <ChevronRight className="ml-auto w-4 h-4" />}
                            </motion.div>
                        </Link>
                    );
                })}
            </nav>

            <div className="p-4 border-t border-border/50">
                <button
                    onClick={() => signOut()}
                    className="w-full flex items-center gap-4 px-6 py-4 rounded-2xl hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-all duration-300 group"
                >
                    <LogOut className="w-5 h-5 group-hover:rotate-12 transition-transform" />
                    <span className="font-medium">Sign Out</span>
                </button>
            </div>

            <div className="p-6">
                <div className="p-6 rounded-2xl bg-gradient-to-br from-primary/20 to-accent/20 border border-primary/10">
                    <p className="text-sm font-medium mb-1">Status</p>
                    <p className="text-xs text-muted-foreground">System active and synced</p>
                </div>
            </div>
        </aside>
    );
};
