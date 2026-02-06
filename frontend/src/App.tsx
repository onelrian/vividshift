import { BrowserRouter as Router, Routes, Route, Navigate } from "react-router-dom";
import { AuthProvider } from "./context/AuthContext";
import { Dashboard } from "./pages/Dashboard";
import { PeopleManagement } from "./pages/PeopleManagement";
import { UserManagement } from "./pages/UserManagement";
import { AssignmentHistory } from "./pages/AssignmentHistory";
import { Settings } from "./pages/Settings";
import { Login } from "./pages/Login";
import { ProtectedRoute } from "./components/ProtectedRoute";

const AppRoutes = () => {
  return (
    <Routes>
      <Route path="/login" element={<Login />} />

      <Route element={<ProtectedRoute />}>
        <Route path="/" element={<Dashboard />} />
        <Route path="/history" element={<AssignmentHistory />} />
        <Route path="/settings" element={<Settings />} />
      </Route>

      <Route element={<ProtectedRoute adminOnly />}>
        <Route path="/people" element={<PeopleManagement />} />
        <Route path="/admin/users" element={<UserManagement />} />
      </Route>

      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
};

function App() {
  return (
    <AuthProvider>
      <Router>
        <AppRoutes />
      </Router>
    </AuthProvider>
  );
}

export default App;
