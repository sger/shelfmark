import { AlertCircle, CheckCircle2, Info, RefreshCcw, X } from 'lucide-react';
import type { ReactNode } from 'react';

type AlertMessageProps = {
  actionLabel?: string;
  children: ReactNode;
  dismissLabel?: string;
  onAction?: () => void;
  onDismiss?: () => void;
  title?: string;
  variant?: 'error' | 'success' | 'info';
};

const icons = {
  error: AlertCircle,
  success: CheckCircle2,
  info: Info,
};

export function AlertMessage({
  actionLabel,
  children,
  dismissLabel = 'Dismiss',
  onAction,
  onDismiss,
  title,
  variant = 'info',
}: AlertMessageProps) {
  const Icon = icons[variant];

  return (
    <div className={`alert-message alert-${variant}`} role={variant === 'error' ? 'alert' : 'status'}>
      <Icon className="alert-icon" size={18} />
      <div className="alert-content">
        {title && <strong>{title}</strong>}
        <span>{children}</span>
      </div>
      {onAction && (
        <button className="alert-action" type="button" onClick={onAction}>
          <RefreshCcw size={14} />
          {actionLabel ?? 'Retry'}
        </button>
      )}
      {onDismiss && (
        <button className="alert-dismiss" type="button" onClick={onDismiss} aria-label={dismissLabel}>
          <X size={14} />
        </button>
      )}
    </div>
  );
}
