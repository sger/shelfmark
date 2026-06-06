import { Link, useParams } from '@tanstack/react-router';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Plus, Save, Star, Trash2 } from 'lucide-react';
import { FormEvent, useEffect, useMemo, useState } from 'react';
import { api } from '../api/client';
import type { BookListItem, ShelfRule, ShelfSummary } from '../api/types';
import { AlertMessage } from '../components/AlertMessage';
import { EmptyState } from '../components/EmptyState';
import { useToast } from '../components/Toast';

type RuleDraft = ShelfRule & { id: string };

const fieldOptions = [
  ['title', 'Title'],
  ['author', 'Author'],
  ['language', 'Language'],
  ['year', 'Published year'],
  ['tag', 'Tag'],
  ['reading_status', 'Reading status'],
  ['favorite', 'Favorite'],
  ['progress_percent', 'Progress percent'],
] as const;

function operatorsFor(field: string) {
  if (field === 'favorite') return [['is', 'is']];
  if (field === 'progress_percent') return [['greater_than', 'greater than'], ['less_than', 'less than']];
  if (field === 'title' || field === 'author') return [['contains', 'contains'], ['equals', 'equals']];
  return [['equals', 'equals']];
}

function newRule(): RuleDraft {
  return { id: crypto.randomUUID(), field: 'title', operator: 'contains', value: '' };
}

export function ShelvesPage() {
  const { shelfId } = useParams({ strict: false }) as { shelfId?: string };
  const queryClient = useQueryClient();
  const { notify } = useToast();
  const shelves = useQuery({ queryKey: ['shelves'], queryFn: () => api<ShelfSummary[]>('/shelves') });
  const selectedShelf = shelves.data?.find((shelf) => shelf.id === shelfId) ?? shelves.data?.[0];
  const selectedId = shelfId ?? selectedShelf?.id;
  const books = useQuery({
    queryKey: ['shelf-books', selectedId],
    queryFn: () => api<BookListItem[]>(`/shelves/${encodeURIComponent(selectedId!)}/books`),
    enabled: Boolean(selectedId),
  });

  const [name, setName] = useState('');
  const [matchMode, setMatchMode] = useState<'all' | 'any'>('all');
  const [rules, setRules] = useState<RuleDraft[]>([newRule()]);
  const isEditing = selectedShelf?.kind === 'custom';

  useEffect(() => {
    if (selectedShelf?.kind === 'custom') {
      setName(selectedShelf.name);
      setMatchMode((selectedShelf.match_mode ?? 'all') as 'all' | 'any');
      setRules((selectedShelf.rules ?? []).map((rule) => ({ ...rule, id: crypto.randomUUID() })));
    }
  }, [selectedShelf?.id, selectedShelf?.kind, selectedShelf?.match_mode, selectedShelf?.name, selectedShelf?.rules]);

  const groupedShelves = useMemo(() => {
    const groups = new Map<string, ShelfSummary[]>();
    for (const shelf of shelves.data ?? []) {
      const group = shelf.group ?? 'other';
      groups.set(group, [...(groups.get(group) ?? []), shelf]);
    }
    return groups;
  }, [shelves.data]);

  const saveShelf = useMutation({
    mutationFn: () => {
      const body = { name, match_mode: matchMode, rules: rules.map(({ id: _id, ...rule }) => rule) };
      if (isEditing && selectedShelf) {
        return api<ShelfSummary>(`/shelves/${selectedShelf.id}`, { method: 'PATCH', body });
      }
      return api<ShelfSummary>('/shelves', { method: 'POST', body });
    },
    onSuccess: async (shelf) => {
      notify({ variant: 'success', title: isEditing ? 'Shelf updated' : 'Shelf created', message: shelf.name });
      await queryClient.invalidateQueries({ queryKey: ['shelves'] });
      await queryClient.invalidateQueries({ queryKey: ['shelf-books'] });
      if (!isEditing) resetForm();
    },
    onError: (error) => notify({ variant: 'error', title: 'Shelf save failed', message: error.message }),
  });

  const deleteShelf = useMutation({
    mutationFn: (id: string) => api<{ ok: boolean }>(`/shelves/${id}`, { method: 'DELETE' }),
    onSuccess: async () => {
      notify({ variant: 'success', title: 'Shelf deleted', message: 'Custom shelf removed.' });
      await queryClient.invalidateQueries({ queryKey: ['shelves'] });
    },
    onError: (error) => notify({ variant: 'error', title: 'Shelf delete failed', message: error.message }),
  });

  function resetForm() {
    setName('');
    setMatchMode('all');
    setRules([newRule()]);
  }

  function submit(event: FormEvent) {
    event.preventDefault();
    saveShelf.mutate();
  }

  function updateRule(id: string, patch: Partial<ShelfRule>) {
    setRules((current) => current.map((rule) => {
      if (rule.id !== id) return rule;
      const next = { ...rule, ...patch };
      if (patch.field) {
        next.operator = operatorsFor(patch.field)[0][0];
        next.value = patch.field === 'favorite' ? false : '';
      }
      return next;
    }));
  }

  return (
    <section className="shelves-layout">
      <aside className="panel shelves-sidebar">
        <div className="panel-header">
          <h2>Smart Shelves</h2>
        </div>
        {shelves.isError && (
          <AlertMessage variant="error" onAction={() => void shelves.refetch()}>
            Unable to load shelves.
          </AlertMessage>
        )}
        {[...groupedShelves.entries()].map(([group, items]) => (
          <div className="shelf-group" key={group}>
            <span>{group.replace('-', ' ')}</span>
            {items.map((shelf) => (
              <Link className={`shelf-link ${selectedId === shelf.id ? 'active' : ''}`} key={shelf.id} to="/shelves/$shelfId" params={{ shelfId: shelf.id }}>
                <strong>{shelf.name}</strong>
                <small>{shelf.count}</small>
              </Link>
            ))}
          </div>
        ))}
      </aside>

      <section className="content-grid">
        <section className="panel">
          <div className="panel-header">
            <div>
              <h2>{selectedShelf?.name ?? 'Shelf books'}</h2>
              <p className="panel-subtitle">{selectedShelf?.count ?? 0} matching books</p>
            </div>
            {selectedShelf?.kind === 'custom' && (
              <button className="icon-button" type="button" onClick={() => deleteShelf.mutate(selectedShelf.id)} disabled={deleteShelf.isPending}>
                <Trash2 size={16} /> Delete
              </button>
            )}
          </div>
          {books.isError ? (
            <AlertMessage variant="error" onAction={() => void books.refetch()}>Unable to load shelf books.</AlertMessage>
          ) : !(books.data ?? []).length ? (
            <EmptyState title="No matching books" detail="Adjust the shelf rules or add matching book state." />
          ) : (
            <div className="book-card-grid">
              {(books.data ?? []).map((book) => (
                <Link className="book-card" key={book.id} to="/books/$bookId" params={{ bookId: book.id }}>
                  <div>
                    <strong>{book.title}</strong>
                    <span>{book.authors.join(', ') || 'Unknown author'}</span>
                  </div>
                  <div className="book-card-meta">
                    <span className={`status-chip status-${book.reading_status}`}>{book.reading_status}</span>
                    {book.favorite && <Star className="favorite-icon" size={15} fill="currentColor" />}
                  </div>
                  <div className="tag-list compact">
                    {book.tags.slice(0, 4).map((tag) => <span className="tag-chip" key={tag}>{tag}</span>)}
                  </div>
                </Link>
              ))}
            </div>
          )}
        </section>

        <form className="panel form-grid" onSubmit={submit}>
          <div className="panel-header">
            <h2>{isEditing ? 'Edit custom shelf' : 'Create custom shelf'}</h2>
            {isEditing && <button className="icon-button" type="button" onClick={resetForm}><Plus size={16} /> New</button>}
          </div>
          <label>Name<input value={name} onChange={(event) => setName(event.target.value)} placeholder="Research books" /></label>
          <label>Match
            <select value={matchMode} onChange={(event) => setMatchMode(event.target.value as 'all' | 'any')}>
              <option value="all">All rules</option>
              <option value="any">Any rule</option>
            </select>
          </label>
          <div className="rule-list">
            {rules.map((rule) => (
              <div className="rule-row" key={rule.id}>
                <select value={rule.field} onChange={(event) => updateRule(rule.id, { field: event.target.value })}>
                  {fieldOptions.map(([value, label]) => <option value={value} key={value}>{label}</option>)}
                </select>
                <select value={rule.operator} onChange={(event) => updateRule(rule.id, { operator: event.target.value })}>
                  {operatorsFor(rule.field).map(([value, label]) => <option value={value} key={value}>{label}</option>)}
                </select>
                {rule.field === 'favorite' ? (
                  <select value={String(rule.value)} onChange={(event) => updateRule(rule.id, { value: event.target.value === 'true' })}>
                    <option value="true">true</option>
                    <option value="false">false</option>
                  </select>
                ) : rule.field === 'reading_status' ? (
                  <select value={String(rule.value || 'unread')} onChange={(event) => updateRule(rule.id, { value: event.target.value })}>
                    <option value="unread">Unread</option>
                    <option value="reading">Reading</option>
                    <option value="finished">Finished</option>
                  </select>
                ) : (
                  <input value={String(rule.value)} type={rule.field === 'progress_percent' ? 'number' : 'text'} onChange={(event) => updateRule(rule.id, { value: rule.field === 'progress_percent' ? Number(event.target.value) : event.target.value })} />
                )}
                <button className="icon-button" type="button" onClick={() => setRules((current) => current.filter((item) => item.id !== rule.id))} disabled={rules.length === 1}>Remove</button>
              </div>
            ))}
          </div>
          <button className="icon-button" type="button" onClick={() => setRules((current) => [...current, newRule()])}><Plus size={16} /> Add rule</button>
          {saveShelf.error && <AlertMessage variant="error" onDismiss={() => saveShelf.reset()}>{saveShelf.error.message}</AlertMessage>}
          <button className="primary-button" type="submit" disabled={saveShelf.isPending}><Save size={16} /> Save shelf</button>
        </form>
      </section>
    </section>
  );
}
