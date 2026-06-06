import { Link } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import { createColumnHelper, flexRender, getCoreRowModel, getSortedRowModel, useReactTable } from '@tanstack/react-table';
import { Search, Star } from 'lucide-react';
import { useMemo, useState } from 'react';
import { api } from '../api/client';
import type { BookListItem } from '../api/types';
import { AlertMessage } from '../components/AlertMessage';
import { EmptyState } from '../components/EmptyState';

const columnHelper = createColumnHelper<BookListItem>();

export function BooksPage() {
  const [query, setQuery] = useState('');
  const books = useQuery({
    queryKey: ['books', query],
    queryFn: () => api<BookListItem[]>(`/books${query ? `?q=${encodeURIComponent(query)}` : ''}`),
  });

  const columns = useMemo(() => [
    columnHelper.accessor('title', {
      header: 'Title',
      cell: (info) => <Link to="/books/$bookId" params={{ bookId: info.row.original.id }}>{info.getValue()}</Link>,
    }),
    columnHelper.accessor((row) => row.authors.join(', ') || 'Unknown', { id: 'authors', header: 'Authors' }),
    columnHelper.accessor('reading_status', { header: 'Status', cell: (info) => <span className={`status-chip status-${info.getValue()}`}>{info.getValue().replace('-', ' ')}</span> }),
    columnHelper.accessor('favorite', { header: 'Fav', cell: (info) => info.getValue() ? <Star className="favorite-icon" size={16} fill="currentColor" /> : '-' }),
    columnHelper.accessor('tags', { header: 'Tags', cell: (info) => <div className="tag-list compact">{info.getValue().slice(0, 3).map((tag) => <span className="tag-chip" key={tag}>{tag}</span>)}</div> }),
    columnHelper.accessor('published_date', { header: 'Published', cell: (info) => info.getValue() ?? '-' }),
    columnHelper.accessor('language', { header: 'Language', cell: (info) => info.getValue() ?? '-' }),
  ], []);

  const table = useReactTable({
    data: books.data ?? [],
    columns,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
  });

  return (
    <section className="panel">
      <div className="panel-header">
        <h2>Books</h2>
        <label className="search-box">
          <Search size={16} />
          <input placeholder="Search title or author" value={query} onChange={(event) => setQuery(event.target.value)} />
        </label>
      </div>
      {books.isError ? (
        <AlertMessage variant="error" onAction={() => void books.refetch()}>
          Unable to load books.
        </AlertMessage>
      ) : !books.data?.length ? (
        <EmptyState title="No books yet" detail="Upload a PDF or EPUB, or scan a library folder." />
      ) : (
        <table className="data-table">
          <thead>
            {table.getHeaderGroups().map((group) => (
              <tr key={group.id}>
                {group.headers.map((header) => (
                  <th key={header.id}>{flexRender(header.column.columnDef.header, header.getContext())}</th>
                ))}
              </tr>
            ))}
          </thead>
          <tbody>
            {table.getRowModel().rows.map((row) => (
              <tr key={row.id}>
                {row.getVisibleCells().map((cell) => (
                  <td key={cell.id}>{flexRender(cell.column.columnDef.cell, cell.getContext())}</td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}
