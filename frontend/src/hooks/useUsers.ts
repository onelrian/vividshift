import { useState } from 'react';
import { useAuth } from '../context/AuthContext';

export interface User {
    id: string;
    username: string;
    email: string;
    role: 'ADMIN' | 'USER';
}

export const useUsers = () => {
    const [users, setUsers] = useState<User[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const { session } = useAuth();

    const fetchUsers = async () => {
        if (!session) return;

        setLoading(true);
        setError(null);

        try {
            const response = await fetch('/api/admin/users', {
                headers: {
                    'Authorization': `Bearer ${session.access_token}`
                }
            });

            if (!response.ok) {
                throw new Error('Failed to fetch users');
            }

            const data = await response.json();
            setUsers(data);
        } catch (err) {
            setError(err instanceof Error ? err.message : 'An error occurred');
        } finally {
            setLoading(false);
        }
    };

    const updateUserRole = async (userId: string, newRole: 'ADMIN' | 'USER') => {
        if (!session) return { success: false, error: 'Not authenticated' };

        try {
            const response = await fetch(`/api/admin/users/${userId}/role`, {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${session.access_token}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ role: newRole })
            });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.error || 'Failed to update user role');
            }

            const updatedUser = await response.json();

            // Update local state
            setUsers(prev => prev.map(u => u.id === userId ? updatedUser : u));

            return { success: true, user: updatedUser };
        } catch (err) {
            const errorMessage = err instanceof Error ? err.message : 'An error occurred';
            return { success: false, error: errorMessage };
        }
    };

    return {
        users,
        loading,
        error,
        fetchUsers,
        updateUserRole
    };
};
