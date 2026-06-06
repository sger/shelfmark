import { Monitor, Moon, Sun } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
import type { ThemePreference } from '../lib/theme';
import { useTheme } from '../lib/theme';

const options: Array<{ icon: LucideIcon; label: string; value: ThemePreference }> = [
  { icon: Monitor, label: 'System', value: 'system' },
  { icon: Sun, label: 'Light', value: 'light' },
  { icon: Moon, label: 'Dark', value: 'dark' },
];

export function SettingsPage() {
  const { resolvedTheme, setTheme, theme } = useTheme();

  return (
    <section className="content-grid">
      <section className="panel settings-panel">
        <div className="panel-header">
          <div>
            <h2>Settings</h2>
            <p className="panel-subtitle">Preferences are saved in this browser.</p>
          </div>
        </div>

        <div className="settings-row">
          <div>
            <strong>Appearance</strong>
            <span>Choose how the interface should render.</span>
          </div>
          <div className="appearance-control">
            <div className="segmented segmented-three" role="radiogroup" aria-label="Appearance">
              {options.map((option) => {
                const Icon = option.icon;
                return (
                  <button
                    key={option.value}
                    type="button"
                    className={theme === option.value ? 'selected' : ''}
                    role="radio"
                    aria-checked={theme === option.value}
                    onClick={() => setTheme(option.value)}
                  >
                    <Icon size={16} />
                    {option.label}
                  </button>
                );
              })}
            </div>
            <span className="settings-hint">
              Active theme: {resolvedTheme === 'dark' ? 'Dark' : 'Light'}
            </span>
          </div>
        </div>
      </section>
    </section>
  );
}
