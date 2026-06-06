import { useQuery } from '@tanstack/react-query';
import { api } from '../api/client';
import type { ImportJob } from '../api/types';
import { AlertMessage } from '../components/AlertMessage';
import { EmptyState } from '../components/EmptyState';
import { StatusBadge } from '../components/StatusBadge';

export function JobsPage() {
  const jobs = useQuery({ queryKey: ['jobs'], queryFn: () => api<ImportJob[]>('/jobs'), refetchInterval: 5000 });

  return (
    <section className="panel">
      <div className="panel-header"><h2>Import jobs</h2></div>
      {jobs.isError && (
        <AlertMessage variant="error" onAction={() => void jobs.refetch()}>
          Unable to load import jobs.
        </AlertMessage>
      )}
      <div className="table">
        {!(jobs.data ?? []).length && !jobs.isError && (
          <EmptyState title="No import jobs" detail="Upload or scan books to start import work." />
        )}
        {(jobs.data ?? []).map((job) => (
          <div className="table-row" key={job.id}>
            <span>{job.kind}</span>
            <StatusBadge status={job.status} />
            <span>{job.message ?? '-'}</span>
            <time>{new Date(job.updated_at).toLocaleString()}</time>
          </div>
        ))}
      </div>
    </section>
  );
}
