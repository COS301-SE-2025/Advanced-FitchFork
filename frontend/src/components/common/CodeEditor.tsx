import { Editor, type OnMount } from '@monaco-editor/react';
import { useRef, useState, useEffect } from 'react';
import { useTheme } from '@/context/ThemeContext';
import { CopyOutlined, CheckOutlined } from '@ant-design/icons';
import { Tooltip, message } from 'antd';

interface CodeEditorProps {
  value: string;
  language?: string;
  onChange?: (value: string | undefined) => void;
  height?: number | string; // accept percentage for fluid height
  readOnly?: boolean;
  className?: string;
  title?: string;
  minimal?: boolean;
  showLineNumbers?: boolean;
  fitContent?: boolean;
  hideCopyButton?: boolean;
}

const CodeEditor: React.FC<CodeEditorProps> = ({
  value,
  language = 'json',
  onChange,
  height = 300,
  readOnly = false,
  className = '',
  title,
  minimal = false,
  showLineNumbers = true,
  fitContent = false,
  hideCopyButton = false,
}) => {
  const { isDarkMode } = useTheme();
  const [copied, setCopied] = useState(false);
  const langLabel = language.toUpperCase();

  const initialHeight = typeof height === 'number' ? `${height}px` : height;
  const [dynamicHeight, setDynamicHeight] = useState<number | string>(
    fitContent ? 0 : initialHeight,
  );

  const editorRef = useRef<import('monaco-editor').editor.IStandaloneCodeEditor | null>(null);
  const disposeRef = useRef<{ dispose: () => void } | null>(null);

  const handleCopy = () => {
    navigator.clipboard.writeText(value || '');
    setCopied(true);
    message.success('Code copied to clipboard');
    setTimeout(() => setCopied(false), 1500);
  };

  const handleMount: OnMount = (editor) => {
    editorRef.current = editor;

    if (fitContent) {
      const applyHeight = () => {
        const contentHeight = editor.getContentHeight();
        setDynamicHeight(contentHeight);
        const { width } = editor.getLayoutInfo();
        editor.layout({ width, height: contentHeight });
      };

      applyHeight();
      disposeRef.current = editor.onDidContentSizeChange(applyHeight);
    }
  };

  useEffect(() => {
    return () => {
      disposeRef.current?.dispose();
      disposeRef.current = null;
      editorRef.current = null;
    };
  }, []);

  const effectiveHeight = fitContent ? dynamicHeight : initialHeight;

  return (
    <div
      className={`relative rounded-md overflow-hidden border border-gray-300 dark:border-gray-700 group
                  flex flex-col min-h-0 ${className}`}
    >
      {!minimal && (
        <div
          className={`flex items-center justify-between px-3 py-2 text-sm font-medium flex-shrink-0
            ${
              isDarkMode
                ? 'bg-gray-800 text-gray-200 border-b border-gray-700'
                : 'bg-gray-100 text-gray-700 border-b border-gray-300'
            }`}
        >
          <span>{title || 'Code Editor'}</span>

          <div className="flex items-center gap-2">
            <span
              className={`px-2 py-0.5 rounded text-xs font-semibold ${
                isDarkMode ? 'bg-gray-700 text-gray-300' : 'bg-gray-200 text-gray-700'
              }`}
            >
              {langLabel}
            </span>

            {!hideCopyButton && (
              <Tooltip title={copied ? 'Copied!' : 'Copy code'}>
                <button
                  type="button"
                  onClick={handleCopy}
                  className="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
                >
                  {copied ? <CheckOutlined /> : <CopyOutlined />}
                </button>
              </Tooltip>
            )}
          </div>
        </div>
      )}

      <div
        className="relative flex-1 min-h-0"
        style={{ height: effectiveHeight as number | string }}
      >
        {minimal && !hideCopyButton && (
          <Tooltip title={copied ? 'Copied!' : 'Copy code'}>
            <button
              type="button"
              onClick={handleCopy}
              className="absolute top-1 right-1 z-10 h-7 w-7 flex items-center justify-center rounded 
                         bg-blue-500 hover:bg-blue-600 
                         opacity-20 group-hover:opacity-100 transition-opacity"
            >
              {copied ? (
                <CheckOutlined className="!text-white text-xs" />
              ) : (
                <CopyOutlined className="!text-white text-xs" />
              )}
            </button>
          </Tooltip>
        )}

        <Editor
          height={effectiveHeight}
          language={language}
          value={value}
          onChange={onChange}
          onMount={handleMount}
          theme={isDarkMode ? 'vs-dark' : 'light'}
          options={{
            readOnly,
            minimap: { enabled: false },
            fontSize: 14,
            wordWrap: 'on',
            scrollBeyondLastLine: false,
            automaticLayout: true,
            // 🔒 Always disable active-line highlight (no prop to re-enable)
            renderLineHighlight: 'none',
            renderLineHighlightOnlyWhenFocus: false,
            // Optional: also avoid extra selection glow/occurrence highlights (keeps things calmer)
            selectionHighlight: false,
            occurrencesHighlight: 'off',

            lineNumbers: showLineNumbers ? 'on' : 'off',
            ...(showLineNumbers
              ? {}
              : {
                  glyphMargin: false,
                  lineDecorationsWidth: 0,
                  lineNumbersMinChars: 0,
                }),
            ...(fitContent
              ? {
                  scrollbar: {
                    vertical: 'hidden',
                    horizontal: 'auto',
                    handleMouseWheel: false,
                    alwaysConsumeMouseWheel: false,
                  },
                  overviewRulerLanes: 0,
                  overviewRulerBorder: false,
                }
              : {}),
          }}
        />
      </div>
    </div>
  );
};

export default CodeEditor;
