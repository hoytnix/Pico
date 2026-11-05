import React, { useState, useEffect, useRef } from 'react';
import { FileText } from 'lucide-react';
import { picoApi, pickImageFileDialog, confirmAction } from '../../api';
import { slugify, formatPostPath } from '../../lib/slug';
import { TiptapEditor } from '../editor/TiptapEditor';
import { PostListSidebar } from './content/PostListSidebar';
import { FrontmatterForm } from './content/FrontmatterForm';
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

  return (
    <div className="content-editor flex h-full bg-neutral-950 text-neutral-100 overflow-hidden">
      {/* Left Sidebar: Collection Tabs & Posts list */}
      <PostListSidebar
        collections={collections}
        activeCollection={activeCollection}
        setActiveCollection={setActiveCollection}
        posts={posts}
        activePostPath={activePostPath}
        setActivePostPath={setActivePostPath}
        filterQuery={filterQuery}
        setFilterQuery={setFilterQuery}
        onCreatePost={handleCreatePost}
        onDeletePost={handleDeletePost}
      />

      {/* Main Authoring Area */}
      <div className="flex-1 flex flex-col min-w-0 overflow-hidden">
        {activeDocument && activePostPath ? (
          <>
            {/* Post Properties Header Form */}
            <FrontmatterForm
              frontmatter={activeDocument.frontmatter}
              saveStatus={saveStatus}
              onChange={handleFrontmatterChange}
            />

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
