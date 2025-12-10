import React from 'react';
import { Navigate, useLocation } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';

const ProtectedRoute = ({ children }) => {
    const { token, user } = useAuth();
    const location = useLocation();

    if (!token) {
        return <Navigate to="/login" state={{ from: location }} replace />;
    }

    // Optional: Add loading state if user is being validated?
    // But token check is synchronous + decode.

    return children;
};

export default ProtectedRoute;
