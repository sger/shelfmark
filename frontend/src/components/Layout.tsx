import { Link, useRouter } from '@tanstack/react-router';
import { BookOpen, Database, Home, Library, ListFilter, LogOut, Settings, Upload, Workflow } from 'lucide-react';
import type { ReactNode } from 'react';
import { useAuth } from '../lib/auth';

const navItems = [
  { to: '/', label: 'Dashboard', icon: Home },
  { to: '/books', label: 'Books', icon: BookOpen },
  { to: '/shelves', label: 'Shelves', icon: ListFilter },
  { to: '/upload', label: 'Upload', icon: Upload },
  { to: '/libraries', label: 'Libraries', icon: Library },
  { to: '/jobs', label: 'Jobs', icon: Workflow },
  { to: '/settings', label: 'Settings', icon: Settings },
] as const;

export function Layout({ children }: { children: ReactNode }) {
  const auth = useAuth();
  const router = useRouter();

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <Database size={22} />
          <span>Shelfmark</span>
        </div>
        <nav className="nav-list">
          {navItems.map((item) => {
            const Icon = item.icon;
            return (
              <Link key={item.to} to={item.to} className="nav-link" activeProps={{ className: 'nav-link active' }}>
                <Icon size={18} />
                <span>{item.label}</span>
              </Link>
            );
          })}
        </nav>
        <button
          className="nav-link logout"
          onClick={() => {
            auth.logout();
            void router.navigate({ to: '/login' });
          }}
        >
          <LogOut size={18} />
          <span>Sign out</span>
        </button>
      </aside>
      <main className="main-panel">
        <header className="topbar">
          <div>
            <p className="eyebrow">Self-hosted library</p>
            <h1>Shelfmark</h1>
          </div>
          <div className="user-pill">{auth.user?.display_name ?? auth.user?.email}</div>
        </header>
        {children}
      </main>
    </div>
  );
}
