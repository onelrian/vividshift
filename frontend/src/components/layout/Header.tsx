import { useState, useEffect } from "react";
import { Sun, Moon, Bell } from "lucide-react";
import { useAuth } from "../../context/AuthContext";

interface HeaderProps {
    title: string;
}

export const Header = ({ title }: HeaderProps) => {
    const { user } = useAuth();
    const [isDarkMode, setIsDarkMode] = useState(false);

    useEffect(() => {
        if (isDarkMode) {
            document.documentElement.classList.add('dark');
        } else {
            document.documentElement.classList.remove('dark');
        }
    }, [isDarkMode]);

    return (
        <header className="h-20 flex items-center justify-between px-8 bg-background/50 backdrop-blur-md border-b border-border/50 sticky top-0 z-10">
            <div className="flex items-center gap-4">
                <h1 className="text-xl font-semibold">{title}</h1>
            </div>

            <div className="flex items-center gap-3">
                <button
                    onClick={() => setIsDarkMode(!isDarkMode)}
                    className="p-3 rounded-xl hover:bg-muted/50 transition-colors border border-border/30"
                >
                    {isDarkMode ? <Sun className="w-5 h-5 text-warning" /> : <Moon className="w-5 h-5 text-accent" />}
                </button>
                <button className="p-3 rounded-xl hover:bg-muted/50 transition-colors border border-border/30 relative">
                    <Bell className="w-5 h-5" />
                    <span className="absolute top-2 right-2 w-2.5 h-2.5 bg-secondary rounded-full border-2 border-background" />
                </button>
                <div className="h-10 w-10 rounded-xl bg-muted border border-border/30 overflow-hidden ml-2 shadow-inner">
                    <img src={`https://api.dicebear.com/7.x/avataaars/svg?seed=${user?.email}`} alt="Avatar" className="w-full h-full object-cover" />
                </div>
            </div>
        </header>
    );
};
