import React, { useMemo, useState, useEffect } from 'react';
import { DiffEditor } from '@monaco-editor/react';
import { Segmented } from 'antd';
import { useTheme } from '@/context/ThemeContext';

type ViewMode = 'side-by-side' | 'inline';

interface CodeDiffEditorProps {
  original: string;
  modified: string;
  language?: string;
  height?: number;
  className?: string;

  /** Compact header (toggle only) when true; full header when false. Default: true */
  minimal?: boolean;

  /** Main title shown only when minimal === false */
  title?: React.ReactNode;

  /** Pane labels (with defaults) */
  leftTitle?: React.ReactNode;
  rightTitle?: React.ReactNode;

  /** Show language badge when minimal === false and language is provided */
  showLangBadge?: boolean;

  /** Controlled mode (optional). If omitted, component manages its own mode state. */
  viewMode?: ViewMode;
  defaultViewMode?: ViewMode;

  /** Callback when view mode changes */
  onViewModeChange?: (mode: ViewMode) => void;
}

const CodeDiffEditor: React.FC<CodeDiffEditorProps> = ({
  original,
  modified,
  language,
  height = 400,
  className = '',
  minimal = true,
  title,
  leftTitle = 'Original',
  rightTitle = 'Modified',
  showLangBadge = true,
  viewMode,
  defaultViewMode = 'side-by-side',
  onViewModeChange,
}) => {
  const { isDarkMode } = useTheme();
  const langLabel = language ? language.toUpperCase() : '';

  const shellBorder = isDarkMode ? 'border-gray-700' : 'border-gray-300';
  const stripBg = isDarkMode
    ? 'bg-gray-900 text-gray-300 border-gray-800'
    : 'bg-gray-50 text-gray-600 border-gray-200';

  const isControlled = viewMode !== undefined;
  const [internalMode, setInternalMode] = useState<ViewMode>(viewMode ?? defaultViewMode);

  useEffect(() => {
    if (isControlled) setInternalMode(viewMode!);
  }, [isControlled, viewMode]);

  const mode = isControlled ? (viewMode as ViewMode) : internalMode;
  const setMode = (m: ViewMode) => {
    if (!isControlled) setInternalMode(m);
    onViewModeChange?.(m);
  };

  const sideBySide = mode === 'side-by-side';

  const options = useMemo(
    () => ({
      renderSideBySide: sideBySide,
      readOnly: true,
      minimap: { enabled: false },
      fontSize: 14,
      scrollBeyondLastLine: false,
      automaticLayout: true,
    }),
    [sideBySide],
  );

  return (
    <div className={`rounded-md overflow-hidden border ${shellBorder} ${className}`}>
      {/* Header */}
      <div
        className={`flex items-center justify-between px-3 py-1.5 text-sm font-medium border-b ${stripBg}`}
      >
        {/* Left: title + optional language badge */}
        {!minimal ? (
          <div className="flex items-center gap-2 min-w-0">
            {title || <span className="truncate">Code Diff</span>}
            {showLangBadge && langLabel && (
              <span
                className={`px-2 py-0.5 rounded text-xs font-semibold ${
                  isDarkMode ? 'bg-gray-700 text-gray-300' : 'bg-gray-200 text-gray-700'
                }`}
              >
                {langLabel}
              </span>
            )}
          </div>
        ) : (
          <span className="text-transparent select-none">.</span>
        )}

        {/* Right: view mode toggle */}
        <Segmented
          size="small"
          value={mode}
          onChange={(v) => setMode(v as ViewMode)}
          options={[
            { label: 'Side-by-side', value: 'side-by-side' },
            { label: 'Inline', value: 'inline' },
          ]}
        />
      </div>

      {/* Pane titles row (only for side-by-side) */}
      {sideBySide && (
        <div className={`grid grid-cols-2 items-center px-3 py-1.5 text-xs border-b ${stripBg}`}>
          <div className="flex items-center min-w-0 gap-2">
            <span
              className={`h-2 w-2 rounded-full ${isDarkMode ? 'bg-red-400/80' : 'bg-red-600/70'}`}
            />
            <span
              className="truncate"
              title={typeof leftTitle === 'string' ? leftTitle : undefined}
            >
              {leftTitle}
            </span>
          </div>
          <div className="flex items-center min-w-0 gap-2 justify-end">
            <span
              className={`h-2 w-2 rounded-full ${isDarkMode ? 'bg-green-400/80' : 'bg-green-600/70'}`}
            />
            <span
              className="truncate text-right"
              title={typeof rightTitle === 'string' ? rightTitle : undefined}
            >
              {rightTitle}
            </span>
          </div>
        </div>
      )}

      {/* Diff Editor */}
      <div style={{ height }}>
        <DiffEditor
          height={height}
          language={language}
          original={original}
          modified={modified}
          theme={isDarkMode ? 'vs-dark' : 'light'}
          options={options}
        />
      </div>
    </div>
  );
};

export default CodeDiffEditor;
