import { useEffect, useState } from 'react';
import { useAuth } from '../context/AuthContext';
import { useUsers } from '../hooks/useUsers';

export const UserManagement = () => {
    const { profile } = useAuth();
    const { users, loading, error, fetchUsers, updateUserRole } = useUsers();
    const [actionLoading, setActionLoading] = useState<string | null>(null);
    const [notification, setNotification] = useState<{ type: 'success' | 'error', message: string } | null>(null);

    useEffect(() => {
        fetchUsers();
    }, []);

    const handleRoleChange = async (userId: string, currentRole: string, email: string) => {
        const newRole = currentRole === 'ADMIN' ? 'USER' : 'ADMIN';
        const action = newRole === 'ADMIN' ? 'promote' : 'demote';

        if (!confirm(`Are you sure you want to ${action} ${email} to ${newRole}?`)) {
            return;
        }

        setActionLoading(userId);
        const result = await updateUserRole(userId, newRole);
        setActionLoading(null);

        if (result.success) {
            setNotification({
                type: 'success',
                message: `Successfully ${action}d ${email} to ${newRole}`
            });
            setTimeout(() => setNotification(null), 3000);
        } else {
            setNotification({
                type: 'error',
                message: result.error || `Failed to ${action} user`
            });
            setTimeout(() => setNotification(null), 5000);
        }
    };

    const isDefaultAdmin = (email: string) => {
        // Check against ADMIN_EMAIL from .env
        const adminEmail = import.meta.env.VITE_ADMIN_EMAIL || 'admin@vividshift.com';
        return email === adminEmail;
    };

    return (
        <div className="container mx-auto px-4 py-8">
            <div className="mb-8">
                <h1 className="text-3xl font-bold text-gray-900 dark:text-white">User Management</h1>
                <p className="mt-2 text-gray-600 dark:text-gray-400">
                    Manage user roles and permissions
                </p>
            </div>

            {notification && (
                <div className={`mb-6 p-4 rounded-lg ${notification.type === 'success'
                        ? 'bg-green-100 text-green-800 border border-green-200'
                        : 'bg-red-100 text-red-800 border border-red-200'
                    }`}>
                    {notification.message}
                </div>
            )}

            {loading && !users.length ? (
                <div className="flex justify-center items-center py-12">
                    <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent"></div>
                </div>
            ) : error ? (
                <div className="bg-red-100 text-red-800 p-4 rounded-lg border border-red-200">
                    Error: {error}
                </div>
            ) : (
                <div className="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden">
                    <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                        <thead className="bg-gray-50 dark:bg-gray-900">
                            <tr>
                                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                                    Email
                                </th>
                                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                                    Username
                                </th>
                                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                                    Role
                                </th>
                                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                                    Actions
                                </th>
                            </tr>
                        </thead>
                        <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                            {users.map((user) => {
                                const isDefault = isDefaultAdmin(user.email);
                                const isCurrentUser = profile?.email === user.email;

                                return (
                                    <tr key={user.id} className="hover:bg-gray-50 dark:hover:bg-gray-700">
                                        <td className="px-6 py-4 whitespace-nowrap">
                                            <div className="flex items-center gap-2">
                                                <span className="text-sm text-gray-900 dark:text-white">
                                                    {user.email}
                                                </span>
                                                {isDefault && (
                                                    <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200">
                                                        Default Admin
                                                    </span>
                                                )}
                                                {isCurrentUser && (
                                                    <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200">
                                                        You
                                                    </span>
                                                )}
                                            </div>
                                        </td>
                                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-white">
                                            {user.username}
                                        </td>
                                        <td className="px-6 py-4 whitespace-nowrap">
                                            <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${user.role === 'ADMIN'
                                                    ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
                                                    : 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200'
                                                }`}>
                                                {user.role}
                                            </span>
                                        </td>
                                        <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                                            <button
                                                onClick={() => handleRoleChange(user.id, user.role, user.email)}
                                                disabled={isDefault || actionLoading === user.id}
                                                className={`px-4 py-2 rounded-lg transition-colors ${isDefault
                                                        ? 'bg-gray-200 text-gray-500 cursor-not-allowed dark:bg-gray-700 dark:text-gray-500'
                                                        : user.role === 'ADMIN'
                                                            ? 'bg-orange-500 text-white hover:bg-orange-600 dark:bg-orange-600 dark:hover:bg-orange-700'
                                                            : 'bg-green-500 text-white hover:bg-green-600 dark:bg-green-600 dark:hover:bg-green-700'
                                                    } ${actionLoading === user.id ? 'opacity-50 cursor-wait' : ''}`}
                                                title={isDefault ? 'Cannot modify default admin' : ''}
                                            >
                                                {actionLoading === user.id ? (
                                                    <span className="flex items-center gap-2">
                                                        <div className="h-4 w-4 animate-spin rounded-full border-2 border-white border-t-transparent"></div>
                                                        Processing...
                                                    </span>
                                                ) : user.role === 'ADMIN' ? (
                                                    'Demote to User'
                                                ) : (
                                                    'Promote to Admin'
                                                )}
                                            </button>
                                        </td>
                                    </tr>
                                );
                            })}
                        </tbody>
                    </table>
                </div>
            )}
        </div>
    );
};
