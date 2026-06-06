import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Upload } from 'lucide-react';
import { FormEvent, useState } from 'react';
import { api } from '../api/client';
import type { Book, Library } from '../api/types';
import { AlertMessage } from '../components/AlertMessage';
import { useToast } from '../components/Toast';

export function UploadPage() {
  const queryClient = useQueryClient();
  const { notify } = useToast();
  const libraries = useQuery({ queryKey: ['libraries'], queryFn: () => api<Library[]>('/libraries') });
  const [file, setFile] = useState<File | null>(null);
  const [libraryId, setLibraryId] = useState('');

  const upload = useMutation({
    mutationFn: async () => {
      if (!file) throw new Error('Choose a file');
      const form = new FormData();
      form.append('file', file);
      if (libraryId) form.append('library_id', libraryId);
      return api<Book>('/books/upload', { method: 'POST', body: form });
    },
    onSuccess: async (book) => {
      setFile(null);
      notify({ variant: 'success', title: 'Book imported', message: book.title });
      await queryClient.invalidateQueries({ queryKey: ['books'] });
      await queryClient.invalidateQueries({ queryKey: ['jobs'] });
    },
    onError: (error) => {
      notify({ variant: 'error', title: 'Upload failed', message: error.message });
    },
  });

  function submit(event: FormEvent) {
    event.preventDefault();
    upload.mutate();
  }

  return (
    <section className="panel">
      <div className="panel-header">
        <h2>Upload book</h2>
      </div>
      <form className="form-grid" onSubmit={submit}>
        <label>
          Library
          <select value={libraryId} onChange={(event) => setLibraryId(event.target.value)}>
            <option value="">Unassigned</option>
            {(libraries.data ?? []).map((library) => (
              <option key={library.id} value={library.id}>{library.name}</option>
            ))}
          </select>
        </label>
        <label>
          PDF or EPUB
          <input type="file" accept=".pdf,.epub,application/pdf,application/epub+zip" onChange={(event) => setFile(event.target.files?.[0] ?? null)} />
        </label>
        {libraries.isError && (
          <AlertMessage variant="error" onAction={() => void libraries.refetch()}>
            Unable to load libraries.
          </AlertMessage>
        )}
        {upload.error && (
          <AlertMessage variant="error" onDismiss={() => upload.reset()}>
            {upload.error.message}
          </AlertMessage>
        )}
        {upload.data && (
          <AlertMessage variant="success" onDismiss={() => upload.reset()}>
            Imported {upload.data.title}
          </AlertMessage>
        )}
        <button className="primary-button" type="submit" disabled={upload.isPending}><Upload size={16} /> Upload</button>
      </form>
    </section>
  );
}
