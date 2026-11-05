import React, { useState, useEffect, useRef } from 'react';
import {
  Plus,
  Trash2,
  FileText,
  Calendar,
  Tag,
  Image as ImageIcon,
  Check,
  Clock,
  AlertTriangle,
} from 'lucide-react';
import { picoApi, pickImageFileDialog, confirmAction } from '../../api';
import { slugify, formatPostPath } from '../../lib/slug';
import { TiptapEditor } from '../editor/TiptapEditor';
import type { ModelDef, PostDocument, PostMeta, SiteInfo } from '../../types';

interface ContentEditorProps {
  currentSite: SiteInfo | null;
  activePostPath: string | null;
  setActivePostPath: (path: string | null) => void;
  onPostSaved?: () => void;
  tiptapEditorRef?: React.MutableRefObject<any>;
}

export function ContentEditor({
  currentSite,
  activePostPath,
  setActivePostPath,
  onPostSaved,
  tiptapEditorRef,
}: ContentEditorProps) {
  const [collections, setCollections] = useState<ModelDef[]>([]);
  const [activeCollection, setActiveCollection] = useState<string>('blog');
  const [posts, setPosts] = useState<PostMeta[]>([]);
  const [activeDocument, setActiveDocument] = useState<PostDocument | null>(null);
  const [saveStatus, setSaveStatus] = useState<'saved' | 'saving' | 'unsaved'>('saved');
  const [filterQuery, setFilterQuery] = useState('');

  const saveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isInitialLoadRef = useRef(true);

  // Load collections
  useEffect(() => {
    if (currentSite?.collections && currentSite.collections.length > 0) {
      setCollections(currentSite.collections);
      if (!currentSite.collections.some((c) => c.name === activeCollection)) {
        setActiveCollection(currentSite.collections[0].name);
      }
    }
  }, [currentSite]);

  // Load content list for active collection
  const refreshPosts = async () => {
    if (!currentSite) return;
    try {
      const items = await picoApi.listContent(activeCollection);
      setPosts(items);
      if (items.length > 0 && !activePostPath) {
        setActivePostPath(items[0].path);
      }
    } catch (err) {
      console.warn('Failed to list content:', err);
    }
  };

  useEffect(() => {
    refreshPosts();
  }, [currentSite, activeCollection]);

  // Load active post document
  useEffect(() => {
    let active = true;
    if (!activePostPath) {
      setActiveDocument(null);
      return;
    }

    async function loadPost() {
      try {
        const doc = await picoApi.readPost(activePostPath!);
        if (active) {
          setActiveDocument(doc);
          setSaveStatus('saved');
        }
      } catch (err) {
        console.warn('Failed to read post:', err);
      }
    }

    loadPost();

    return () => {
      active = false;
    };
  }, [activePostPath]);

  // Handle Save
  const saveCurrentPost = async (docToSave?: PostDocument) => {
    const doc = docToSave || activeDocument;
    if (!activePostPath || !doc) return;
    setSaveStatus('saving');
    try {
      await picoApi.savePost(activePostPath, doc.frontmatter, doc.markdown);
      setSaveStatus('saved');
      onPostSaved?.();
      refreshPosts();
    } catch (err) {
      console.warn('Failed to save post:', err);
      setSaveStatus('unsaved');
    }
  };

  // Debounced auto-save
  const triggerAutoSave = (updatedDoc: PostDocument) => {
    setActiveDocument(updatedDoc);
    setSaveStatus('unsaved');
    if (saveTimerRef.current) clearTimeout(saveTimerRef.current);
    saveTimerRef.current = setTimeout(() => {
      saveCurrentPost(updatedDoc);
    }, 1500);
  };

  // Update frontmatter property
  const handleFrontmatterChange = (key: string, value: any) => {
    if (!activeDocument) return;
    const updated = {
      ...activeDocument,
      frontmatter: {
        ...activeDocument.frontmatter,
        [key]: value,
      },
    };
    triggerAutoSave(updated);
  };

  // Update markdown body from TipTap
  const handleMarkdownChange = (newMarkdown: string) => {
    if (!activeDocument) return;
    const updated = {
      ...activeDocument,
      markdown: newMarkdown,
    };
    triggerAutoSave(updated);
  };

  // Create new post
  const handleCreatePost = async () => {
    const title = window.prompt('Enter post title:');
    if (!title || !title.trim()) return;
    const slug = slugify(title) || 'new-post';
    const dateStr = new Date().toISOString().split('T')[0];
    const path = formatPostPath(activeCollection, slug);

    const initialFrontmatter: Record<string, unknown> = {
      title: title.trim(),
      date: dateStr,
      slug,
      draft: true,
      tags: [],
      excerpt: '',
    };

    try {
      await picoApi.savePost(path, initialFrontmatter, `# ${title.trim()}\n\nWrite your content here...`);
      await refreshPosts();
      setActivePostPath(path);
    } catch (err: any) {
      alert(`Failed to create post: ${err.message || String(err)}`);
    }
  };

  // Delete post
  const handleDeletePost = async (path: string, title: string) => {
    const confirmed = await confirmAction(`Are you sure you want to delete "${title}"?`);
    if (!confirmed) return;
    try {
      await picoApi.deletePost(path);
      if (activePostPath === path) {
        setActivePostPath(null);
      }
      refreshPosts();
    } catch (err: any) {
      alert(`Failed to delete post: ${err.message || String(err)}`);
    }
  };

  // Pick and insert image
  const handlePickImage = async () => {
    const filePath = await pickImageFileDialog();
    if (!filePath || !activePostPath) return;

    try {
      const slug = (activeDocument?.frontmatter?.slug as string) || 'post';
      const relativeAssetPath = await picoApi.saveImage(filePath, slug, activeCollection);

      // Insert markdown image into editor
      if (tiptapEditorRef?.current) {
        tiptapEditorRef.current.commands.setImage({
          src: relativeAssetPath,
          alt: slug,
        });
      }
    } catch (err: any) {
      alert(`Failed to insert image: ${err.message || String(err)}`);
    }
  };

  const currentModel = collections.find((c) => c.name === activeCollection);
  const filteredPosts = posts.filter((p) =>
    (p.title || p.slug).toLowerCase().includes(filterQuery.toLowerCase())
  );

  return (
    <div className="content-editor flex h-full bg-neutral-950 text-neutral-100 overflow-hidden">
      {/* Left Sidebar: Collection Tabs & Posts list */}
      <div className="w-64 border-r border-neutral-800 bg-neutral-900/60 flex flex-col shrink-0">
        {/* Collection Selector Tabs */}
        {collections.length > 1 && (
          <div className="flex border-b border-neutral-800 p-1.5 gap-1 bg-neutral-900">
            {collections.map((col) => (
              <button
                key={col.name}
                type="button"
                onClick={() => {
                  setActiveCollection(col.name);
                  setActivePostPath(null);
                }}
                className={`flex-1 py-1 px-2 text-xs font-medium rounded-md truncate transition-colors cursor-pointer ${
                  activeCollection === col.name
                    ? 'bg-indigo-600 text-white shadow-xs'
                    : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800'
                }`}
              >
                {col.label || col.name}
              </button>
            ))}
          </div>
        )}

        {/* Search & New Post Button */}
        <div className="p-2.5 border-b border-neutral-800 flex items-center gap-2">
          <input
            type="text"
            value={filterQuery}
            onChange={(e) => setFilterQuery(e.target.value)}
            placeholder="Filter posts…"
            className="flex-1 bg-neutral-950 border border-neutral-700 rounded-md px-2.5 py-1 text-xs text-neutral-200 outline-none focus:border-indigo-500"
          />
          <button
            type="button"
            onClick={handleCreatePost}
            className="p-1.5 rounded-md bg-indigo-600 hover:bg-indigo-500 text-white transition-colors cursor-pointer"
            title="New Post"
          >
            <Plus className="w-3.5 h-3.5" />
          </button>
        </div>

        {/* Posts List */}
        <div className="flex-1 overflow-y-auto p-2 space-y-1">
          {filteredPosts.length === 0 ? (
            <div className="text-center text-xs text-neutral-500 py-6">No posts found</div>
          ) : (
            filteredPosts.map((post) => (
              <div
                key={post.path}
                onClick={() => setActivePostPath(post.path)}
                className={`group flex items-start justify-between p-2.5 rounded-lg text-xs cursor-pointer transition-colors ${
                  activePostPath === post.path
                    ? 'bg-indigo-600/20 text-indigo-300 border border-indigo-500/40'
                    : 'hover:bg-neutral-800/80 text-neutral-300'
                }`}
              >
                <div className="flex-1 min-w-0 pr-2">
                  <div className="font-medium truncate">{post.title || post.slug}</div>
                  <div className="flex items-center gap-2 text-[10px] text-neutral-400 mt-0.5">
                    <span>{post.date || 'No date'}</span>
                    {post.draft && (
                      <span className="px-1 py-0.2 rounded bg-amber-950/80 text-amber-400 border border-amber-800/60">
                        Draft
                      </span>
                    )}
                  </div>
                </div>
                <button
                  type="button"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleDeletePost(post.path, post.title || post.slug);
                  }}
                  className="opacity-0 group-hover:opacity-100 p-1 hover:text-red-400 text-neutral-500 transition-opacity"
                  title="Delete post"
                >
                  <Trash2 className="w-3 h-3" />
                </button>
              </div>
            ))
          )}
        </div>
      </div>

      {/* Main Authoring Area */}
      <div className="flex-1 flex flex-col min-w-0 overflow-hidden">
        {activeDocument && activePostPath ? (
          <>
            {/* Post Properties Header Form */}
            <div className="p-4 border-b border-neutral-800 bg-neutral-900/40">
              <div className="flex items-center justify-between mb-3">
                <input
                  type="text"
                  value={(activeDocument.frontmatter.title as string) || ''}
                  onChange={(e) => handleFrontmatterChange('title', e.target.value)}
                  placeholder="Post Title"
                  className="text-lg font-bold text-neutral-100 bg-transparent outline-none flex-1 mr-4 focus:border-b focus:border-indigo-500"
                />
                <div className="flex items-center gap-3 text-xs">
                  {saveStatus === 'saving' && (
                    <span className="text-amber-400 flex items-center gap-1">
                      <Clock className="w-3.5 h-3.5 animate-spin" /> Saving…
                    </span>
                  )}
                  {saveStatus === 'saved' && (
                    <span className="text-emerald-400 flex items-center gap-1">
                      <Check className="w-3.5 h-3.5" /> Saved
                    </span>
                  )}
                  {saveStatus === 'unsaved' && (
                    <span className="text-amber-400 flex items-center gap-1">
                      Unsaved changes
                    </span>
                  )}
                  <label className="flex items-center gap-1.5 cursor-pointer text-neutral-300">
                    <input
                      type="checkbox"
                      checked={Boolean(activeDocument.frontmatter.draft)}
                      onChange={(e) => handleFrontmatterChange('draft', e.target.checked)}
                      className="rounded bg-neutral-800 border-neutral-700 text-indigo-600 focus:ring-0"
                    />
                    <span>Draft</span>
                  </label>
                </div>
              </div>

              {/* Secondary Properties Row */}
              <div className="grid grid-cols-3 gap-3 text-xs">
                <div>
                  <label className="block text-[11px] text-neutral-400 mb-1">Date</label>
                  <input
                    type="date"
                    value={String(activeDocument.frontmatter.date || '')}
                    onChange={(e) => handleFrontmatterChange('date', e.target.value)}
                    className="w-full bg-neutral-950 border border-neutral-700 rounded px-2.5 py-1 text-neutral-200 outline-none focus:border-indigo-500"
                  />
                </div>
                <div>
                  <label className="block text-[11px] text-neutral-400 mb-1">Slug</label>
                  <input
                    type="text"
                    value={(activeDocument.frontmatter.slug as string) || ''}
                    onChange={(e) => handleFrontmatterChange('slug', e.target.value)}
                    className="w-full bg-neutral-950 border border-neutral-700 rounded px-2.5 py-1 text-neutral-200 font-mono outline-none focus:border-indigo-500"
                  />
                </div>
                <div>
                  <label className="block text-[11px] text-neutral-400 mb-1">Tags (comma-separated)</label>
                  <input
                    type="text"
                    value={
                      Array.isArray(activeDocument.frontmatter.tags)
                        ? activeDocument.frontmatter.tags.join(', ')
                        : String(activeDocument.frontmatter.tags || '')
                    }
                    onChange={(e) => {
                      const tagsArray = e.target.value
                        .split(',')
                        .map((t) => t.trim())
                        .filter(Boolean);
                      handleFrontmatterChange('tags', tagsArray);
                    }}
                    className="w-full bg-neutral-950 border border-neutral-700 rounded px-2.5 py-1 text-neutral-200 outline-none focus:border-indigo-500"
                  />
                </div>
              </div>
            </div>

            {/* TipTap WYSIWYG Editor Container */}
            <div className="flex-1 p-4 overflow-hidden flex flex-col">
              <TiptapEditor
                markdown={activeDocument.markdown}
                postPath={activePostPath}
                onUpdateMarkdown={handleMarkdownChange}
                onPickImage={handlePickImage}
                setEditorInstance={(ed) => {
                  if (tiptapEditorRef) tiptapEditorRef.current = ed;
                }}
              />
            </div>
          </>
        ) : (
          <div className="flex-1 flex flex-col items-center justify-center text-neutral-500 text-sm">
            <FileText className="w-10 h-10 mb-2 opacity-40" />
            <p>Select a post from the left sidebar to edit, or create a new post.</p>
          </div>
        )}
      </div>
    </div>
  );
}
