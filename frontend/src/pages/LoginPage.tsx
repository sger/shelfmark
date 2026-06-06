import { useRouter } from '@tanstack/react-router';
import { BookMarked } from 'lucide-react';
import { FormEvent, useState } from 'react';
import { AlertMessage } from '../components/AlertMessage';
import { useToast } from '../components/Toast';
import { useAuth } from '../lib/auth';

export function LoginPage() {
  const auth = useAuth();
  const router = useRouter();
  const { notify } = useToast();
  const [mode, setMode] = useState<'login' | 'register'>('login');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [displayName, setDisplayName] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function submit(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setError(null);
    try {
      if (mode === 'register') {
        await auth.register(email, password, displayName);
        notify({ variant: 'success', title: 'Account created', message: 'Your library is ready.' });
      } else {
        await auth.login(email, password);
        notify({ variant: 'success', title: 'Signed in', message: 'Welcome back.' });
      }
      await router.navigate({ to: '/' });
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Authentication failed';
      setError(message);
      notify({ variant: 'error', title: mode === 'register' ? 'Account creation failed' : 'Sign in failed', message });
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="login-screen">
      <section className="login-panel">
        <div className="brand large">
          <BookMarked size={28} />
          <span>Shelfmark</span>
        </div>
        <div className="segmented">
          <button type="button" className={mode === 'login' ? 'selected' : ''} onClick={() => setMode('login')}>Sign in</button>
          <button type="button" className={mode === 'register' ? 'selected' : ''} onClick={() => setMode('register')}>Create account</button>
        </div>
        <form onSubmit={submit} className="form-grid">
          {mode === 'register' && (
            <label>
              Display name
              <input value={displayName} onChange={(event) => setDisplayName(event.target.value)} required />
            </label>
          )}
          <label>
            Email
            <input type="email" value={email} onChange={(event) => setEmail(event.target.value)} required />
          </label>
          <label>
            Password
            <input type="password" value={password} onChange={(event) => setPassword(event.target.value)} minLength={8} required />
          </label>
          {error && (
            <AlertMessage variant="error" onDismiss={() => setError(null)}>
              {error}
            </AlertMessage>
          )}
          <button className="primary-button" type="submit" disabled={busy}>
            {busy ? 'Working...' : mode === 'register' ? 'Create account' : 'Sign in'}
          </button>
        </form>
      </section>
    </main>
  );
}
