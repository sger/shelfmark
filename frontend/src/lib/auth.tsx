import React, { createContext, useCallback, useContext, useMemo, useState } from 'react';
import { api, setAuthToken } from '../api/client';
import type { AuthResponse, User } from '../api/types';

type AuthState = {
  token: string | null;
  user: User | null;
  login: (email: string, password: string) => Promise<void>;
  register: (email: string, password: string, displayName: string) => Promise<void>;
  logout: () => void;
};

const AuthContext = createContext<AuthState | null>(null);

const storedToken = localStorage.getItem('auth_token');
const storedUser = localStorage.getItem('auth_user');
if (storedToken) setAuthToken(storedToken);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [token, setToken] = useState<string | null>(storedToken);
  const [user, setUser] = useState<User | null>(storedUser ? JSON.parse(storedUser) : null);

  const applyAuth = useCallback((response: AuthResponse) => {
    localStorage.setItem('auth_token', response.token);
    localStorage.setItem('auth_user', JSON.stringify(response.user));
    setAuthToken(response.token);
    setToken(response.token);
    setUser(response.user);
  }, []);

  const login = useCallback(async (email: string, password: string) => {
    applyAuth(await api<AuthResponse>('/auth/login', { method: 'POST', body: { email, password } }));
  }, [applyAuth]);

  const register = useCallback(async (email: string, password: string, displayName: string) => {
    applyAuth(await api<AuthResponse>('/auth/register', {
      method: 'POST',
      body: { email, password, display_name: displayName },
    }));
  }, [applyAuth]);

  const logout = useCallback(() => {
    localStorage.removeItem('auth_token');
    localStorage.removeItem('auth_user');
    setAuthToken(null);
    setToken(null);
    setUser(null);
  }, []);

  const value = useMemo(() => ({ token, user, login, register, logout }), [token, user, login, register, logout]);
  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const value = useContext(AuthContext);
  if (!value) throw new Error('useAuth must be used inside AuthProvider');
  return value;
}

