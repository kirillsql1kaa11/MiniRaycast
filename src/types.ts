export interface LauncherItem {
  id: string;
  title: string;
  subtitle: string;
  item_type: string;
  action: string;
  payload: string;
  badge?: string;
  keywords?: string[];
}

export interface AppearanceSettings {
  theme: string;
  opacity: string;
  blur: string;
}

export interface HistoryRecord {
  id: number;
  query: string;
  executed_at: string;
}

export interface AppUsageRecord {
  item_id: string;
  title: string;
  path: string;
  item_type: string;
  launch_count: number;
}
