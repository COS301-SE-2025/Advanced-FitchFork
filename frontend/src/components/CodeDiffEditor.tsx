import { DiffEditor } from '@monaco-editor/react';
import { useTheme } from '@/context/ThemeContext';

interface CodeDiffEditorProps {
  original: string;
  modified: string;
  language?: string;
  height?: number;
  className?: string;
  title?: string;
}

const CodeDiffEditor: React.FC<CodeDiffEditorProps> = ({
  original,
  modified,
  language = 'json',
  height = 400,
  className = '',
  title,
}) => {
  const { isDarkMode } = useTheme();

  const langLabel = language.toUpperCase();

  return (
    <div
      className={`rounded-md overflow-hidden border border-gray-300 dark:border-gray-700 ${className}`}
    >
      {/* Header */}
      <div
        className={`flex items-center justify-between px-3 py-2 text-sm font-medium ${
          isDarkMode
            ? 'bg-gray-800 text-gray-200 border-b border-gray-700'
            : 'bg-gray-100 text-gray-700 border-b border-gray-300'
        }`}
      >
        <span>{title || 'Code Diff Viewer'}</span>

        <span
          className={`px-2 py-0.5 rounded text-xs font-semibold ${
            isDarkMode ? 'bg-gray-700 text-gray-300' : 'bg-gray-200 text-gray-700'
          }`}
        >
          {langLabel}
        </span>
      </div>

      {/* Fixed-height Diff Editor */}
      <div
        style={{
          height,
          overflow: 'hidden',
        }}
      >
        <DiffEditor
          height={height}
          language={language}
          original={original}
          modified={modified}
          theme={isDarkMode ? 'vs-dark' : 'light'}
          options={{
            renderSideBySide: true,
            readOnly: true,
            minimap: { enabled: false },
            fontSize: 14,
            scrollBeyondLastLine: false,
            automaticLayout: true,
          }}
        />
      </div>
    </div>
  );
};

export default CodeDiffEditor;
