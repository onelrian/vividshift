import { createContext, useContext, useEffect, useState } from 'react';
import type { ReactNode } from 'react';

interface User {
    email: string;
}

interface Session {
    access_token: string;
}

interface AuthContextType {
    session: Session | null;
    user: User | null;
    loading: boolean;
    signIn: (email: string, password: string) => Promise<void>;
    signOut: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const AuthProvider = ({ children }: { children: ReactNode }) => {
    const [session, setSession] = useState<Session | null>(null);
    const [user, setUser] = useState<User | null>(null);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        const savedSession = localStorage.getItem('vividshift_session');
        if (savedSession) {
            const parsed = JSON.parse(savedSession);
            setSession(parsed);
            setUser({ email: 'admin@vividshift.io' }); // Simple fallback for now
        }
        setLoading(false);
    }, []);

    const signIn = async (email: string, password: string) => {
        const resp = await fetch('/api/auth/login', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ email, password }),
        });

        if (!resp.ok) {
            throw new Error('Invalid credentials');
        }

        const data = await resp.json();
        const newSession = { access_token: data.token };
        setSession(newSession);
        setUser({ email });
        localStorage.setItem('vividshift_session', JSON.stringify(newSession));
    };

    const signOut = () => {
        setSession(null);
        setUser(null);
        localStorage.removeItem('vividshift_session');
    };

    return (
        <AuthContext.Provider value={{ session, user, loading, signIn, signOut }}>
            {children}
        </AuthContext.Provider>
    );
};

export const useAuth = () => {
    const context = useContext(AuthContext);
    if (context === undefined) {
        throw new Error('useAuth must be used within an AuthProvider');
    }
    return context;
};
