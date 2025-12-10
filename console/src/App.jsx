import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import Dashboard from '@/pages/Dashboard';
import Tenants from '@/pages/Tenants';
import Services from '@/pages/Services';
import Policies from '@/pages/Policies';
import LiveLogs from '@/pages/LiveLogs';

import Login from '@/pages/Login';
import { AuthProvider } from '@/context/AuthContext';
import ProtectedRoute from '@/components/ProtectedRoute';

function App() {
  return (
    <AuthProvider>
      <Router>
        <Routes>
          <Route path="/login" element={<Login />} />
          <Route
            path="/*"
            element={
              <ProtectedRoute>
                <Routes>
                  <Route path="/" element={<Dashboard />} />
                  <Route path="/dashboard" element={<Dashboard />} />
                  <Route path="/tenants" element={<Tenants />} />
                  <Route path="/services" element={<Services />} />
                  <Route path="/policies" element={<Policies />} />
                  <Route path="/logs" element={<LiveLogs />} />
                </Routes>
              </ProtectedRoute>
            }
          />
        </Routes>
      </Router>
    </AuthProvider>
  );
}

export default App;
