import { useQuery } from '@tanstack/react-query';
import { BookOpen, Library, Workflow } from 'lucide-react';
import { api } from '../api/client';
import type { Book, ImportJob, Library as LibraryType } from '../api/types';
import { AlertMessage } from '../components/AlertMessage';
import { EmptyState } from '../components/EmptyState';
import { StatusBadge } from '../components/StatusBadge';

export function DashboardPage() {
  const books = useQuery({ queryKey: ['books'], queryFn: () => api<Book[]>('/books') });
  const libraries = useQuery({ queryKey: ['libraries'], queryFn: () => api<LibraryType[]>('/libraries') });
  const jobs = useQuery({ queryKey: ['jobs'], queryFn: () => api<ImportJob[]>('/jobs') });

  const cards = [
    { label: 'Books', value: books.data?.length ?? 0, icon: BookOpen },
    { label: 'Libraries', value: libraries.data?.length ?? 0, icon: Library },
    { label: 'Recent jobs', value: jobs.data?.length ?? 0, icon: Workflow },
  ];

  return (
    <section className="content-grid">
      <div className="metric-row">
        {cards.map((card) => {
          const Icon = card.icon;
          return (
            <article className="metric-card" key={card.label}>
              <Icon size={20} />
              <span>{card.label}</span>
              <strong>{card.value}</strong>
            </article>
          );
        })}
      </div>
      <section className="panel">
        <div className="panel-header">
          <h2>Recent import activity</h2>
        </div>
        {(books.isError || libraries.isError || jobs.isError) && (
          <AlertMessage
            variant="error"
            onAction={() => {
              void books.refetch();
              void libraries.refetch();
              void jobs.refetch();
            }}
          >
            Some dashboard data could not be loaded.
          </AlertMessage>
        )}
        <div className="table">
          {!(jobs.data ?? []).length && !jobs.isError && (
            <EmptyState title="No import activity" detail="Upload a book or scan a folder to create import jobs." />
          )}
          {(jobs.data ?? []).slice(0, 8).map((job) => (
            <div className="table-row" key={job.id}>
              <span>{job.kind}</span>
              <StatusBadge status={job.status} />
              <span>{job.message ?? 'No message'}</span>
              <time>{new Date(job.updated_at).toLocaleString()}</time>
            </div>
          ))}
        </div>
      </section>
    </section>
  );
}
