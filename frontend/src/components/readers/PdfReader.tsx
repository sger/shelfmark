import { Viewer, Worker } from '@react-pdf-viewer/core';
import type { DocumentLoadEvent, PageChangeEvent } from '@react-pdf-viewer/core';
import { pageNavigationPlugin } from '@react-pdf-viewer/page-navigation';
import { toolbarPlugin } from '@react-pdf-viewer/toolbar';
import { useState } from 'react';
import workerUrl from 'pdfjs-dist/build/pdf.worker.min.js?url';

import '@react-pdf-viewer/core/lib/styles/index.css';
import '@react-pdf-viewer/page-navigation/lib/styles/index.css';
import '@react-pdf-viewer/toolbar/lib/styles/index.css';

type PdfReaderProps = {
  fileUrl: string;
  initialPosition?: string | null;
  onProgress: (position: string, progressPercent: number) => void;
};

function pageFromPosition(position?: string | null) {
  const match = position?.match(/^pdf-page:(\d+)$/);
  if (!match) return 0;
  return Math.max(Number(match[1]) - 1, 0);
}

export function PdfReader({ fileUrl, initialPosition, onProgress }: PdfReaderProps) {
  const [pageCount, setPageCount] = useState(0);
  const initialPage = pageFromPosition(initialPosition);
  const toolbarPluginInstance = toolbarPlugin();
  const pageNavigationPluginInstance = pageNavigationPlugin();
  const { Toolbar } = toolbarPluginInstance;

  function handleDocumentLoad(event: DocumentLoadEvent) {
    setPageCount(event.doc.numPages);
  }

  function handlePageChange(event: PageChangeEvent) {
    const currentPage = event.currentPage + 1;
    const progressPercent = pageCount > 0 ? (currentPage / pageCount) * 100 : 0;
    onProgress(`pdf-page:${currentPage}`, Math.round(progressPercent * 10) / 10);
  }

  return (
    <div className="reader-format-panel">
      <div className="pdf-toolbar">
        <Toolbar />
      </div>
      <div className="pdf-viewer-region">
        <Worker workerUrl={workerUrl}>
          <Viewer
            fileUrl={fileUrl}
            initialPage={initialPage}
            onDocumentLoad={handleDocumentLoad}
            onPageChange={handlePageChange}
            plugins={[toolbarPluginInstance, pageNavigationPluginInstance]}
          />
        </Worker>
      </div>
    </div>
  );
}
