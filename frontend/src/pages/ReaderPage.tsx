import { Link, useParams } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import { ArrowLeft } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { api, authHeaders, fileUrl } from '../api/client';
import type { Book, ReadingProgress } from '../api/types';
import { AlertMessage } from '../components/AlertMessage';
import { EpubReader } from '../components/readers/EpubReader';
import { PdfReader } from '../components/readers/PdfReader';
import { useToast } from '../components/Toast';

export function ReaderPage() {
  const { bookId } = useParams({ strict: false }) as { bookId: string };
  const { notify } = useToast();
  const book = useQuery({ queryKey: ['book', bookId], queryFn: () => api<Book>(`/books/${bookId}`) });
  const progress = useQuery({
    queryKey: ['progress', bookId],
    queryFn: () => api<ReadingProgress | null>(`/books/${bookId}/progress`),
  });
  const [blobUrl, setBlobUrl] = useState<string | null>(null);
  const [mediaType, setMediaType] = useState<string | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);

  const handleProgress = useCallback((position: string, progressPercent: number) => {
    void api<ReadingProgress>(`/books/${bookId}/progress`, {
      method: 'PATCH',
      body: { position, progress_percent: progressPercent },
    }).catch((error: unknown) => {
      notify({
        variant: 'error',
        title: 'Progress was not saved',
        message: error instanceof Error ? error.message : 'Reader progress update failed.',
      });
    });
  }, [bookId, notify]);

  useEffect(() => {
    let active = true;
    let nextBlobUrl: string | null = null;

    setBlobUrl(null);
    setMediaType(null);
    setLoadError(null);

    fetch(fileUrl(bookId), { headers: authHeaders() })
      .then((response) => {
        if (!response.ok) throw new Error(`Unable to load file: ${response.status}`);
        return response.blob();
      })
      .then((blob) => {
        if (!active) return;
        nextBlobUrl = URL.createObjectURL(blob);
        setMediaType(blob.type);
        setBlobUrl(nextBlobUrl);
      })
      .catch((err: unknown) => {
        if (!active) return;
        const message = err instanceof Error ? err.message : 'Unable to load book file';
        setLoadError(message);
        notify({ variant: 'error', title: 'Reader failed to load', message });
      });

    return () => {
      active = false;
      if (nextBlobUrl) URL.revokeObjectURL(nextBlobUrl);
    };
  }, [bookId]);

  const isPdf = mediaType === 'application/pdf';
  const isEpub = mediaType === 'application/epub+zip' || mediaType === 'application/octet-stream';

  return (
    <section className="reader-shell">
      <header className="reader-toolbar">
        <Link className="icon-button" to="/books/$bookId" params={{ bookId }}>
          <ArrowLeft size={16} />
          Back
        </Link>
        <strong>{book.data?.title ?? 'Reader'}</strong>
        <span className="reader-progress-pill">
          {progress.data ? `${Math.round(progress.data.progress_percent)}%` : 'Not started'}
        </span>
      </header>

      {book.isError && (
        <div className="reader-overlay">
          <AlertMessage variant="error" onAction={() => void book.refetch()}>
            Unable to load book details.
          </AlertMessage>
        </div>
      )}
      {loadError && <div className="reader-overlay"><AlertMessage variant="error">{loadError}</AlertMessage></div>}
      {!loadError && !blobUrl && <div className="reader-overlay">Loading book...</div>}
      {blobUrl && isPdf && (
        <PdfReader
          fileUrl={blobUrl}
          initialPosition={progress.data?.position}
          onProgress={handleProgress}
        />
      )}
      {blobUrl && !isPdf && isEpub && (
        <EpubReader
          fileUrl={blobUrl}
          initialPosition={progress.data?.position}
          onProgress={handleProgress}
        />
      )}
      {blobUrl && !isPdf && !isEpub && (
        <div className="reader-overlay"><AlertMessage variant="error">Unsupported reader format: {mediaType || 'unknown'}</AlertMessage></div>
      )}
    </section>
  );
}
