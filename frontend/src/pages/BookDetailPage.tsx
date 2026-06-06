import { Link, useParams } from '@tanstack/react-router';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { BookOpen, Save, Star } from 'lucide-react';
import { FormEvent, useEffect, useState } from 'react';
import { api } from '../api/client';
import type { Book, BookUserState, MetadataResult, ReadingStatus } from '../api/types';
import { AlertMessage } from '../components/AlertMessage';
import { useToast } from '../components/Toast';

export function BookDetailPage() {
  const { bookId } = useParams({ strict: false }) as { bookId: string };
  const queryClient = useQueryClient();
  const { notify } = useToast();
  const book = useQuery({ queryKey: ['book', bookId], queryFn: () => api<Book>(`/books/${bookId}`) });
  const bookState = useQuery({ queryKey: ['book-state', bookId], queryFn: () => api<BookUserState>(`/books/${bookId}/state`) });
  const [stateForm, setStateForm] = useState<{ favorite: boolean; reading_status: ReadingStatus; tags: string }>({ favorite: false, reading_status: 'unread', tags: '' });
  const [form, setForm] = useState({ title: '', authors: '', description: '', publisher: '', published_date: '', language: '', page_count: '' });
  const [metadataQuery, setMetadataQuery] = useState('');

  useEffect(() => {
    if (bookState.data) {
      setStateForm({
        favorite: bookState.data.favorite,
        reading_status: bookState.data.reading_status,
        tags: bookState.data.tags.join(', '),
      });
    }
  }, [bookState.data]);

  useEffect(() => {
    if (book.data) {
      setForm({
        title: book.data.title,
        authors: book.data.authors.join(', '),
        description: book.data.description ?? '',
        publisher: book.data.publisher ?? '',
        published_date: book.data.published_date ?? '',
        language: book.data.language ?? '',
        page_count: book.data.page_count?.toString() ?? '',
      });
      setMetadataQuery(book.data.title);
    }
  }, [book.data]);

  const save = useMutation({
    mutationFn: () => api<Book>(`/books/${bookId}`, {
      method: 'PATCH',
      body: {
        ...form,
        authors: form.authors.split(',').map((author) => author.trim()).filter(Boolean),
        page_count: form.page_count ? Number(form.page_count) : null,
      },
    }),
    onSuccess: async () => {
      notify({ variant: 'success', title: 'Metadata saved', message: form.title || 'Book details updated.' });
      await queryClient.invalidateQueries({ queryKey: ['book', bookId] });
      await queryClient.invalidateQueries({ queryKey: ['books'] });
    },
    onError: (error) => {
      notify({ variant: 'error', title: 'Save failed', message: error.message });
    },
  });

  const stateSave = useMutation({
    mutationFn: () => api<BookUserState>(`/books/${bookId}/state`, {
      method: 'PATCH',
      body: {
        favorite: stateForm.favorite,
        reading_status: stateForm.reading_status,
        tags: stateForm.tags.split(',').map((tag) => tag.trim()).filter(Boolean),
      },
    }),
    onSuccess: async () => {
      notify({ variant: 'success', title: 'Book state saved', message: 'Shelf membership updated.' });
      await queryClient.invalidateQueries({ queryKey: ['book-state', bookId] });
      await queryClient.invalidateQueries({ queryKey: ['books'] });
      await queryClient.invalidateQueries({ queryKey: ['shelves'] });
    },
    onError: (error) => notify({ variant: 'error', title: 'State save failed', message: error.message }),
  });

  const metadata = useQuery({
    queryKey: ['metadata', metadataQuery],
    queryFn: () => api<MetadataResult[]>(`/metadata/open-library/search?q=${encodeURIComponent(metadataQuery)}`),
    enabled: metadataQuery.length > 2,
  });

  function submit(event: FormEvent) {
    event.preventDefault();
    save.mutate();
  }

  function applyMetadata(result: MetadataResult) {
    setForm({
      title: result.title,
      authors: result.authors.join(', '),
      description: result.description ?? form.description,
      publisher: result.publisher ?? '',
      published_date: result.published_date ?? '',
      language: result.language ?? '',
      page_count: result.page_count?.toString() ?? '',
    });
    notify({ variant: 'info', title: 'Metadata applied', message: 'Review and save the changes.' });
  }

  return (
    <section className="detail-grid">
      <form className="panel form-grid" onSubmit={submit}>
        <div className="panel-header">
          <h2>Edit metadata</h2>
          <Link className="icon-button" to="/books/$bookId/read" params={{ bookId }}>
            <BookOpen size={18} />
            Read
          </Link>
        </div>
        {book.isError && (
          <AlertMessage variant="error" onAction={() => void book.refetch()}>
            Unable to load book details.
          </AlertMessage>
        )}
        <label>Title<input value={form.title} onChange={(event) => setForm({ ...form, title: event.target.value })} /></label>
        <label>Authors<input value={form.authors} onChange={(event) => setForm({ ...form, authors: event.target.value })} /></label>
        <label>Description<textarea value={form.description} onChange={(event) => setForm({ ...form, description: event.target.value })} /></label>
        <label>Publisher<input value={form.publisher} onChange={(event) => setForm({ ...form, publisher: event.target.value })} /></label>
        <label>Published date<input value={form.published_date} onChange={(event) => setForm({ ...form, published_date: event.target.value })} /></label>
        <label>Language<input value={form.language} onChange={(event) => setForm({ ...form, language: event.target.value })} /></label>
        <label>Page count<input type="number" value={form.page_count} onChange={(event) => setForm({ ...form, page_count: event.target.value })} /></label>
        {save.error && (
          <AlertMessage variant="error" onDismiss={() => save.reset()}>
            {save.error.message}
          </AlertMessage>
        )}
        {save.isSuccess && (
          <AlertMessage variant="success" onDismiss={() => save.reset()}>
            Metadata saved.
          </AlertMessage>
        )}
        <button className="primary-button" type="submit" disabled={save.isPending}><Save size={16} /> Save</button>
      </form>
      <aside className="panel form-grid">
        <div className="panel-header">
          <h2>Book state</h2>
          <button className="icon-button" type="button" onClick={() => setStateForm({ ...stateForm, favorite: !stateForm.favorite })}>
            <Star size={16} fill={stateForm.favorite ? 'currentColor' : 'none'} />
            {stateForm.favorite ? 'Favorite' : 'Mark favorite'}
          </button>
        </div>
        {bookState.isError && (
          <AlertMessage variant="error" onAction={() => void bookState.refetch()}>
            Unable to load book state.
          </AlertMessage>
        )}
        <label>Status
          <select value={stateForm.reading_status} onChange={(event) => setStateForm({ ...stateForm, reading_status: event.target.value as ReadingStatus })}>
            <option value="unread">Unread</option>
            <option value="reading">Reading</option>
            <option value="finished">Finished</option>
          </select>
        </label>
        <label>Tags
          <input value={stateForm.tags} onChange={(event) => setStateForm({ ...stateForm, tags: event.target.value })} placeholder="fiction, research, work" />
        </label>
        <button className="primary-button" type="button" disabled={stateSave.isPending} onClick={() => stateSave.mutate()}>
          <Save size={16} /> Save state
        </button>
      </aside>
      <aside className="panel">
        <div className="panel-header">
          <h2>Open Library</h2>
        </div>
        <label className="form-grid">Search<input value={metadataQuery} onChange={(event) => setMetadataQuery(event.target.value)} /></label>
        {metadata.isError && (
          <AlertMessage variant="error" onAction={() => void metadata.refetch()}>
            Open Library search failed.
          </AlertMessage>
        )}
        <div className="result-list">
          {(metadata.data ?? []).map((result) => (
            <button className="metadata-result" key={`${result.title}-${result.published_date}`} onClick={() => applyMetadata(result)}>
              <strong>{result.title}</strong>
              <span>{result.authors.join(', ') || 'Unknown author'}</span>
              <small>{result.published_date ?? ''}</small>
            </button>
          ))}
        </div>
      </aside>
    </section>
  );
}
