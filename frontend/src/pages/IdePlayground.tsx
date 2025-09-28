import { useMemo, useRef, useState, useEffect, useCallback } from 'react';
import { Layout, Tree, Tabs, Button, Tooltip, Space, Dropdown, Typography, Drawer } from 'antd';
import type { MenuProps } from 'antd';
import {
  FolderOpenOutlined,
  FileTextOutlined,
  PlusOutlined,
  SaveOutlined,
  MenuOutlined,
} from '@ant-design/icons';
import Editor, { type BeforeMount, type OnMount } from '@monaco-editor/react';
import clsx from 'clsx';
import { useTheme } from '@/context/ThemeContext';
import { useUI } from '@/context/UIContext';

// Types
export type VFile = { id: string; name: string; path: string[]; language: string; value: string };
export type IdePlaygroundProps = { readOnly?: boolean; files?: VFile[] };

// Helpers
const pathToKey = (path: string[], name?: string) => (name ? [...path, name] : path).join('/');

function useTreeData(files: VFile[]) {
  type Node = { key: string; title: string; icon?: React.ReactNode; children?: Node[] };
  const root: Record<string, Node> = {};
  const leaves: Node[] = [];

  const ensure = (segments: string[]) => {
    let curKey = '',
      curMap = root;
    let parent: Node | undefined;
    for (const seg of segments) {
      curKey = curKey ? `${curKey}/${seg}` : seg;
      if (!curMap[curKey])
        curMap[curKey] = { key: curKey, title: seg, icon: <FolderOpenOutlined />, children: [] };
      parent = curMap[curKey];
      curMap =
        (curMap as any)[`${curKey}::children`] ?? ((curMap as any)[`${curKey}::children`] = {});
    }
    return parent!;
  };

  for (const f of files) {
    if (f.path.length) ensure(f.path);
    leaves.push({ key: pathToKey(f.path, f.name), title: f.name, icon: <FileTextOutlined /> });
  }

  const mapToArray = (map: Record<string, Node>): Node[] =>
    Object.entries(map)
      .filter(([k]) => !k.endsWith('::children'))
      .map(([k, node]) => {
        const childrenMap = (map as any)[`${k}::children`] as Record<string, Node> | undefined;
        const children = childrenMap ? mapToArray(childrenMap) : [];
        const folderFiles = leaves.filter(
          (l) => l.key.startsWith(`${k}/`) && !l.key.slice(k.length + 1).includes('/'),
        );
        return { ...node, children: [...children, ...folderFiles] };
      });

  const topFiles = leaves.filter((l) => !l.key.includes('/'));
  return [...mapToArray(root), ...topFiles];
}

// Component
export default function IdePlayground({
  readOnly = false,
  files: filesProp = [],
}: IdePlaygroundProps) {
  const { isDarkMode } = useTheme();
  const { isMobile } = useUI();

  const [files, setFiles] = useState<VFile[]>(filesProp);
  useEffect(() => setFiles(filesProp), [filesProp]);

  const [openIds, setOpenIds] = useState<string[]>([]);
  const [activeId, setActiveId] = useState<string>();
  const [treeOpen, setTreeOpen] = useState(false);

  const monacoRef = useRef<any>(null);
  const editorRef = useRef<any>(null);

  const treeData = useTreeData(files);

  const idByPath = useMemo(() => {
    const map: Record<string, string> = {};
    for (const f of files) map[pathToKey(f.path, f.name)] = f.id;
    return map;
  }, [files]);

  const openFileById = useCallback(
    (id: string) => {
      if (!openIds.includes(id)) setOpenIds((s) => [...s, id]);
      setActiveId(id);
      if (isMobile) setTreeOpen(false);
    },
    [openIds, isMobile],
  );

  const openFileByKey = useCallback(
    (key: string) => {
      const id = idByPath[key];
      if (id) openFileById(id);
    },
    [idByPath, openFileById],
  );

  const createFile = () => {
    if (readOnly) return;
    const nf: VFile = {
      id: crypto.randomUUID(),
      name: `untitled-${files.length + 1}.ts`,
      path: [],
      language: 'typescript',
      value: '// new file\n',
    };
    setFiles((s) => [...s, nf]);
    openFileById(nf.id);
  };

  const closeTab = (targetId: string) => {
    const next = openIds.filter((id) => id !== targetId);
    setOpenIds(next);
    if (activeId === targetId) setActiveId(next[0]);
  };

  const saveAll = () => {
    /* no-op */
  };

  // Monaco themes
  const beforeMount: BeforeMount = (monaco) => {
    monaco.editor.defineTheme('ff-light', {
      base: 'vs',
      inherit: true,
      rules: [],
      colors: {
        'editor.background': '#ffffff',
        'editorGutter.background': '#ffffff',
        'editorLineNumber.activeForeground': '#1677ff',
      },
    });
    monaco.editor.defineTheme('ff-dark', {
      base: 'vs-dark',
      inherit: true,
      rules: [],
      colors: {
        'editor.background': '#141414',
        'editorGutter.background': '#141414',
        'editorLineNumber.activeForeground': '#1677ff',
      },
    });
  };

  const onEditorMount: OnMount = (editor, monaco) => {
    editorRef.current = editor;
    monacoRef.current = monaco;
    monaco.editor.setTheme(isDarkMode ? 'ff-dark' : 'ff-light');
    editor.layout();
  };

  useEffect(() => {
    if (monacoRef.current) {
      monacoRef.current.editor.setTheme(isDarkMode ? 'ff-dark' : 'ff-light');
      editorRef.current?.layout();
    }
  }, [isDarkMode]);

  // Toolbar menu (no export)
  const items: MenuProps['items'] = [
    { key: 'new', icon: <PlusOutlined />, label: 'New file', onClick: createFile },
    {
      key: 'save',
      icon: <SaveOutlined />,
      label: 'Save all',
      onClick: saveAll,
      disabled: !openIds.length,
    },
  ];

  // File tree renderer; header only on desktop sider
  const renderFileTree = (withHeader: boolean) => (
    <div className={withHeader ? '!bg-white dark:!bg-gray-900 h-full flex flex-col' : undefined}>
      {withHeader && (
        <div className="p-3 border-b border-gray-200 dark:border-gray-800 flex justify-between items-center">
          <Typography.Text strong>Files</Typography.Text>
          {!readOnly && <Button size="small" icon={<PlusOutlined />} onClick={createFile} />}
        </div>
      )}
      <div className={withHeader ? 'pt-2 pb-3 overflow-auto flex-1' : undefined}>
        <Tree
          showIcon
          defaultExpandAll
          treeData={treeData}
          onSelect={(k) => openFileByKey(String(k[0]))}
        />
      </div>
    </div>
  );

  return (
    <Layout className="!h-full !bg-gray-50 dark:!bg-gray-950">
      {/* Header / Toolbar */}
      <Layout.Header className="flex items-center justify-between !bg-white dark:!bg-gray-900 border-b border-gray-200 dark:border-gray-800 !px-1 sm:!px-2 !py-0">
        <Space size={8}>
          {isMobile && <Button icon={<MenuOutlined />} onClick={() => setTreeOpen(true)} />}
          {!readOnly && (
            <Dropdown menu={{ items }} trigger={['click']}>
              <Button type="primary" icon={<PlusOutlined />}>
                New
              </Button>
            </Dropdown>
          )}
        </Space>
        <Space size={8} wrap>
          {!readOnly && (
            <Tooltip title={openIds.length ? '' : 'Open a file to enable'}>
              <Button icon={<SaveOutlined />} onClick={saveAll} disabled={!openIds.length}>
                Save
              </Button>
            </Tooltip>
          )}
        </Space>
      </Layout.Header>

      <Layout className="!bg-transparent !h-full">
        {/* Desktop Sider with header */}
        {!isMobile && (
          <Layout.Sider
            width={220}
            className="!bg-white dark:!bg-gray-900 border-r border-gray-200 dark:border-gray-800"
          >
            {renderFileTree(true)}
          </Layout.Sider>
        )}

        {/* Mobile Drawer: use Drawer title and show RAW Tree (no internal header) */}
        {isMobile && (
          <Drawer
            title="Files"
            placement="left"
            width={260}
            open={treeOpen}
            onClose={() => setTreeOpen(false)}
          >
            {renderFileTree(false)}
          </Drawer>
        )}

        {/* Editor / Tabs */}
        <Layout.Content className={clsx('flex flex-col h-full', isDarkMode ? 'dark' : undefined)}>
          {openIds.length ? (
            <Tabs
              type="editable-card"
              size={isMobile ? 'small' : 'middle'}
              hideAdd={readOnly}
              activeKey={activeId}
              animated
              onChange={(k) => setActiveId(String(k))}
              onEdit={(targetKey, action) => action === 'remove' && closeTab(String(targetKey))}
              className="bg-gray-100 dark:bg-gray-950 flex-1 flex flex-col !h-full [&_.ant-tabs-content]:!h-full [&_.ant-tabs-tabpane]:!h-full"
              tabBarStyle={{
                marginTop: isMobile ? 2 : 7,
                marginBottom: 0,
                paddingLeft: isMobile ? 2 : 4,
                paddingRight: isMobile ? 2 : 4,
              }}
              items={openIds.map((id) => {
                const f = files.find((x) => x.id === id)!;
                return {
                  key: id,
                  label: (
                    <span className="flex items-center gap-2">
                      <FileTextOutlined />
                      <span className="max-w-[40vw] sm:max-w-[20vw] truncate">{f.name}</span>
                    </span>
                  ),
                  closable: true,
                  children: (
                    <Editor
                      theme={isDarkMode ? 'ff-dark' : 'ff-light'}
                      language={f.language}
                      value={f.value}
                      beforeMount={beforeMount}
                      onMount={onEditorMount}
                      onChange={(val) => {
                        if (!readOnly) {
                          setFiles((prev) =>
                            prev.map((x) => (x.id === f.id ? { ...x, value: val ?? '' } : x)),
                          );
                        }
                      }}
                      options={{
                        readOnly,
                        readOnlyMessage: { value: '🔒 Read-only preview' },
                        minimap: { enabled: !isMobile },
                        fontSize: isMobile ? 13 : 14,
                        automaticLayout: true,
                        scrollBeyondLastLine: false,
                        tabSize: 2,
                        padding: { top: isMobile ? 8 : 12, bottom: isMobile ? 8 : 12 },
                      }}
                    />
                  ),
                };
              })}
            />
          ) : (
            <div className="flex-1 grid place-items-center p-4 sm:p-6 text-center">
              <div className="w-full max-w-lg bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-xl p-6">
                <div className="flex items-start justify-between mb-4">
                  <div>
                    <Typography.Title level={4} className="!mt-0 !mb-1">
                      No file open
                    </Typography.Title>
                    <Typography.Paragraph type="secondary" className="!mb-0">
                      {isMobile
                        ? 'Tap the menu to open the file tree.'
                        : 'Choose a file from the tree on the left, or start with one of the suggestions below.'}
                    </Typography.Paragraph>
                  </div>
                  <FolderOpenOutlined className="text-gray-400 dark:text-gray-500 text-2xl" />
                </div>

                {/* Quick suggestions */}
                {files.length > 0 && (
                  <div className="mb-5 text-left">
                    <Typography.Text strong className="block mb-2">
                      Suggested files
                    </Typography.Text>
                    <div className="flex flex-wrap gap-2">
                      {files
                        .sort((a, b) => {
                          const pri = (n: string) =>
                            n.toLowerCase() === 'readme.md'
                              ? 1000
                              : n.toLowerCase().startsWith('index.')
                                ? 900
                                : n.toLowerCase().startsWith('main.')
                                  ? 800
                                  : n.endsWith('.md')
                                    ? 200
                                    : 100;
                          return pri(b.name) - pri(a.name);
                        })
                        .slice(0, 5)
                        .map((f) => (
                          <Button
                            key={f.id}
                            size="small"
                            icon={<FileTextOutlined />}
                            onClick={() => openFileById(f.id)}
                          >
                            {f.path.length ? `${f.path.join('/')}/` : ''}
                            {f.name}
                          </Button>
                        ))}
                    </div>
                  </div>
                )}

                {/* Actions */}
                <div className="flex flex-wrap justify-center gap-2">
                  {!readOnly && (
                    <Button type="primary" icon={<PlusOutlined />} onClick={createFile}>
                      Create file
                    </Button>
                  )}
                </div>
              </div>
            </div>
          )}
        </Layout.Content>
      </Layout>
    </Layout>
  );
}
