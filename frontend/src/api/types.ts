export type User = {
  id: string;
  email: string;
  display_name: string;
  role: 'admin' | 'user';
  created_at: string;
};

export type AuthResponse = {
  token: string;
  user: User;
};

export type Library = {
  id: string;
  name: string;
  path: string;
  created_at: string;
  updated_at: string;
};

export type Book = {
  id: string;
  library_id: string | null;
  title: string;
  authors: string[];
  description: string | null;
  publisher: string | null;
  published_date: string | null;
  language: string | null;
  page_count: number | null;
  cover_path: string | null;
  created_at: string;
  updated_at: string;
};

export type ReadingStatus = 'unread' | 'reading' | 'finished';

export type BookListItem = Book & {
  favorite: boolean;
  reading_status: ReadingStatus;
  tags: string[];
  progress_percent: number | null;
};

export type BookUserState = {
  user_id: string;
  book_id: string;
  favorite: boolean;
  reading_status: ReadingStatus;
  tags: string[];
  updated_at: string;
};

export type ShelfRule = {
  field: string;
  operator: string;
  value: string | number | boolean;
};

export type ShelfSummary = {
  id: string;
  name: string;
  kind: 'built-in' | 'custom';
  count: number;
  group: string | null;
  rules: ShelfRule[] | null;
  match_mode: 'all' | 'any' | null;
};

export type ImportJob = {
  id: string;
  kind: 'upload' | 'scan';
  status: 'queued' | 'running' | 'completed' | 'failed';
  message: string | null;
  library_id: string | null;
  book_id: string | null;
  created_at: string;
  updated_at: string;
};

export type ReadingProgress = {
  id: string;
  user_id: string;
  book_id: string;
  position: string;
  progress_percent: number;
  updated_at: string;
};

export type MetadataResult = {
  title: string;
  authors: string[];
  description: string | null;
  publisher: string | null;
  published_date: string | null;
  language: string | null;
  page_count: number | null;
  cover_url: string | null;
};

