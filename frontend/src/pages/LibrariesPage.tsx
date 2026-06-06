import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { FolderSearch, Plus } from 'lucide-react';
import { FormEvent, useState } from 'react';
import { api } from '../api/client';
import type { ImportJob, Library } from '../api/types';
import { AlertMessage } from '../components/AlertMessage';
import { EmptyState } from '../components/EmptyState';
import { useToast } from '../components/Toast';

export function LibrariesPage() {
  const queryClient = useQueryClient();
  const { notify } = useToast();
  const libraries = useQuery({ queryKey: ['libraries'], queryFn: () => api<Library[]>('/libraries') });
  const [name, setName] = useState('');
  const [path, setPath] = useState('/books');

  const create = useMutation({
    mutationFn: () => api<Library>('/libraries', { method: 'POST', body: { name, path } }),
    onSuccess: async (library) => {
      setName('');
      notify({ variant: 'success', title: 'Library added', message: library.name });
      await queryClient.invalidateQueries({ queryKey: ['libraries'] });
    },
    onError: (error) => {
      notify({ variant: 'error', title: 'Library was not added', message: error.message });
    },
  });

  const scan = useMutation({
    mutationFn: (id: string) => api<ImportJob>(`/libraries/${id}/scan`, { method: 'POST' }),
    onSuccess: async () => {
      notify({ variant: 'success', title: 'Scan started', message: 'Import jobs will update as files are checked.' });
      await queryClient.invalidateQueries({ queryKey: ['jobs'] });
    },
    onError: (error) => {
      notify({ variant: 'error', title: 'Scan failed', message: error.message });
    },
  });

  function submit(event: FormEvent) {
    event.preventDefault();
    create.mutate();
  }

  return (
    <section className="two-column">
      <form className="panel form-grid" onSubmit={submit}>
        <div className="panel-header"><h2>Add library</h2></div>
        <label>Name<input value={name} onChange={(event) => setName(event.target.value)} /></label>
        <label>Folder path<input value={path} onChange={(event) => setPath(event.target.value)} /></label>
        {create.error && (
          <AlertMessage variant="error" onDismiss={() => create.reset()}>
            {create.error.message}
          </AlertMessage>
        )}
        <button className="primary-button" type="submit" disabled={create.isPending}><Plus size={16} /> Add library</button>
      </form>
      <section className="panel">
        <div className="panel-header"><h2>Configured folders</h2></div>
        {libraries.isError && (
          <AlertMessage variant="error" onAction={() => void libraries.refetch()}>
            Unable to load configured folders.
          </AlertMessage>
        )}
        <div className="library-list">
          {!libraries.isError && !(libraries.data ?? []).length && (
            <EmptyState title="No folders configured" detail="Add a mounted folder path to scan local books." />
          )}
          {(libraries.data ?? []).map((library) => (
            <article className="library-item" key={library.id}>
              <div>
                <strong>{library.name}</strong>
                <span>{library.path}</span>
              </div>
              <button className="icon-button" onClick={() => scan.mutate(library.id)} disabled={scan.isPending}>
                <FolderSearch size={16} />
                Scan
              </button>
            </article>
          ))}
        </div>
      </section>
    </section>
  );
}
