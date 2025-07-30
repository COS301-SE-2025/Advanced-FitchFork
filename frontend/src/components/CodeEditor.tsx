import { Editor } from '@monaco-editor/react';
import { useState } from 'react';
import { useTheme } from '@/context/ThemeContext';
import { CopyOutlined, CheckOutlined } from '@ant-design/icons';
import { Tooltip, message } from 'antd';

interface CodeEditorProps {
  value: string;
  language?: string;
  onChange?: (value: string | undefined) => void;
  height?: number;
  readOnly?: boolean;
  className?: string;
  title?: string;
  minimal?: boolean;
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
}) => {
  const { isDarkMode } = useTheme();
  const [copied, setCopied] = useState(false);

  const langLabel = language.toUpperCase();

  const handleCopy = () => {
    navigator.clipboard.writeText(value || '');
    setCopied(true);
    message.success('Code copied to clipboard');
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <div
      className={`relative rounded-md overflow-hidden border border-gray-300 dark:border-gray-700 group ${className}`}
    >
      {/* Standard header */}
      {!minimal && (
        <div
          className={`flex items-center justify-between px-3 py-2 text-sm font-medium ${
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

            <Tooltip title={copied ? 'Copied!' : 'Copy code'}>
              <button
                type="button"
                onClick={handleCopy}
                className="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
              >
                {copied ? <CheckOutlined /> : <CopyOutlined />}
              </button>
            </Tooltip>
          </div>
        </div>
      )}

      {/* Editor wrapper */}
      <div style={{ height }} className="relative">
        {/* Minimal floating copy button */}
        {minimal && (
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
          height={height}
          language={language}
          value={value}
          onChange={onChange}
          theme={isDarkMode ? 'vs-dark' : 'light'}
          options={{
            readOnly,
            minimap: { enabled: false },
            fontSize: 14,
            wordWrap: 'on',
            scrollBeyondLastLine: false,
            automaticLayout: true,
          }}
        />
      </div>
    </div>
  );
};

export default CodeEditor;
