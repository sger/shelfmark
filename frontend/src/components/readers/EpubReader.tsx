import ePub from 'epubjs';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import { AlertMessage } from '../AlertMessage';

type EpubReaderProps = {
  fileUrl: string;
  initialPosition?: string | null;
  onProgress: (position: string, progressPercent: number) => void;
};

export function EpubReader({ fileUrl, initialPosition, onProgress }: EpubReaderProps) {
  const viewerRef = useRef<HTMLDivElement | null>(null);
  const renditionRef = useRef<any>(null);
  const [progressLabel, setProgressLabel] = useState('0%');
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!viewerRef.current) return;

    setLoading(true);
    setError(null);
    viewerRef.current.innerHTML = '';

    const book = ePub(fileUrl);
    const rendition = book.renderTo(viewerRef.current, {
      height: '100%',
      spread: 'none',
      width: '100%',
    });

    renditionRef.current = rendition;

    book.ready
      .then(() => book.locations.generate(1200))
      .catch(() => undefined)
      .finally(() => {
        rendition
          .display(initialPosition || undefined)
          .then(() => setLoading(false))
          .catch((err: unknown) => {
            setLoading(false);
            setError(err instanceof Error ? err.message : 'Unable to open EPUB');
          });
      });

    rendition.on('relocated', (location: any) => {
      const position = location?.start?.cfi;
      if (!position) return;

      const rawPercent = book.locations?.percentageFromCfi
        ? book.locations.percentageFromCfi(position) * 100
        : 0;
      const progressPercent = Math.max(0, Math.min(100, Math.round(rawPercent * 10) / 10));
      setProgressLabel(`${progressPercent}%`);
      onProgress(position, progressPercent);
    });

    return () => {
      renditionRef.current = null;
      rendition.destroy();
      book.destroy();
    };
  }, [fileUrl, initialPosition, onProgress]);

  return (
    <div className="reader-format-panel">
      <div className="epub-toolbar">
        <button className="icon-button" onClick={() => renditionRef.current?.prev()}>
          <ChevronLeft size={16} />
          Prev
        </button>
        <span className="reader-progress-pill">{progressLabel}</span>
        <button className="icon-button" onClick={() => renditionRef.current?.next()}>
          Next
          <ChevronRight size={16} />
        </button>
      </div>
      <div className="epub-viewer-region">
        {loading && <div className="reader-overlay">Loading EPUB...</div>}
        {error && <div className="reader-overlay"><AlertMessage variant="error">{error}</AlertMessage></div>}
        <div ref={viewerRef} className="epub-frame" />
      </div>
    </div>
  );
}
