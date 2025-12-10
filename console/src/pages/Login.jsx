import React, { useState } from 'react';
import { useAuth } from '../context/AuthContext';
import { useNavigate } from 'react-router-dom';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Button } from "@/components/ui/button"
import { Label } from "@/components/ui/label"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { ShieldCheck, User, Lock } from 'lucide-react';

const Login = () => {
    const [username, setUsername] = useState('');
    const [password, setPassword] = useState('');
    const [error, setError] = useState('');
    const { login } = useAuth();
    const navigate = useNavigate();

    const handleSubmit = async (e) => {
        e.preventDefault();
        setError('');
        const success = await login(username, password);
        if (success) {
            navigate('/');
        } else {
            setError('Invalid username or password');
        }
    };

    return (
        <div className="flex items-center justify-center min-h-screen bg-background relative overflow-hidden">
            {/* Gradient Background matching Dashboard */}
            <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-primary/20 via-background to-background pointer-events-none" />

            {/* Glass Card */}
            <Card className="w-[400px] z-10 glass-card border-none shadow-2xl bg-black/40 backdrop-blur-2xl">
                <CardHeader className="text-center pb-2">
                    <div className="mx-auto w-12 h-12 bg-primary/20 rounded-xl flex items-center justify-center mb-4">
                        <ShieldCheck className="w-6 h-6 text-primary" />
                    </div>
                    <CardTitle className="text-2xl font-bold tracking-tight">Welcome Back</CardTitle>
                    <CardDescription className="text-muted-foreground/80">Enter your credentials to access the control plane</CardDescription>
                </CardHeader>
                <CardContent>
                    <form onSubmit={handleSubmit} className="space-y-6">
                        <div className="space-y-2">
                            <Label htmlFor="username">Username</Label>
                            <div className="relative">
                                <User className="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
                                <Input
                                    id="username"
                                    className="pl-10 bg-background/50 border-input/50 focus:bg-background transition-colors"
                                    value={username}
                                    onChange={(e) => setUsername(e.target.value)}
                                    placeholder="Enter your username"
                                    required
                                />
                            </div>
                        </div>
                        <div className="space-y-2">
                            <Label htmlFor="password">Password</Label>
                            <div className="relative">
                                <Lock className="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
                                <Input
                                    id="password"
                                    type="password"
                                    className="pl-10 bg-background/50 border-input/50 focus:bg-background transition-colors"
                                    value={password}
                                    onChange={(e) => setPassword(e.target.value)}
                                    placeholder="••••••••"
                                    required
                                />
                            </div>
                        </div>
                        {error && (
                            <Alert variant="destructive" className="bg-destructive/10 border-destructive/20 text-destructive">
                                <AlertDescription>{error}</AlertDescription>
                            </Alert>
                        )}
                        <Button type="submit" className="w-full bg-primary hover:bg-primary/90 text-primary-foreground font-semibold h-11 shadow-lg shadow-primary/20 transition-all">
                            Sign In
                        </Button>
                    </form>
                </CardContent>
            </Card>
        </div>
    );
};

export default Login;
