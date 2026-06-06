import { AlertCircle, CheckCircle2, Info, X } from 'lucide-react';
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from 'react';

type ToastVariant = 'error' | 'success' | 'info';

type Toast = {
  id: string;
  message: string;
  title?: string;
  variant: ToastVariant;
};

type ToastInput = Omit<Toast, 'id'>;

type ToastContextValue = {
  notify: (toast: ToastInput) => void;
  remove: (id: string) => void;
};

const ToastContext = createContext<ToastContextValue | null>(null);

const icons = {
  error: AlertCircle,
  success: CheckCircle2,
  info: Info,
};

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const remove = useCallback((id: string) => {
    setToasts((current) => current.filter((toast) => toast.id !== id));
  }, []);

  const notify = useCallback((toast: ToastInput) => {
    const id = crypto.randomUUID();
    setToasts((current) => [...current.slice(-3), { ...toast, id }]);
  }, []);

  const value = useMemo(() => ({ notify, remove }), [notify, remove]);

  return (
    <ToastContext.Provider value={value}>
      {children}
      <div className="toast-viewport" aria-live="polite" aria-relevant="additions">
        {toasts.map((toast) => (
          <ToastItem key={toast.id} toast={toast} onRemove={remove} />
        ))}
      </div>
    </ToastContext.Provider>
  );
}

function ToastItem({ toast, onRemove }: { toast: Toast; onRemove: (id: string) => void }) {
  const Icon = icons[toast.variant];

  useEffect(() => {
    const timeout = window.setTimeout(() => onRemove(toast.id), toast.variant === 'error' ? 6500 : 4200);
    return () => window.clearTimeout(timeout);
  }, [onRemove, toast.id, toast.variant]);

  return (
    <div className={`toast toast-${toast.variant}`} role={toast.variant === 'error' ? 'alert' : 'status'}>
      <Icon className="toast-icon" size={18} />
      <div className="toast-copy">
        {toast.title && <strong>{toast.title}</strong>}
        <span>{toast.message}</span>
      </div>
      <button className="toast-close" type="button" onClick={() => onRemove(toast.id)} aria-label="Dismiss">
        <X size={14} />
      </button>
    </div>
  );
}

export function useToast() {
  const context = useContext(ToastContext);
  if (!context) throw new Error('useToast must be used inside ToastProvider');
  return context;
}
