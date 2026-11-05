import React from 'react';
import { Plus, Trash2 } from 'lucide-react';
import type { ModelDef, PostMeta } from '../../../types';

interface PostListSidebarProps {
  collections: ModelDef[];
  activeCollection: string;
  setActiveCollection: (col: string) => void;
  posts: PostMeta[];
  activePostPath: string | null;
  setActivePostPath: (path: string | null) => void;
  filterQuery: string;
  setFilterQuery: (query: string) => void;
  onCreatePost: () => void;
  onDeletePost: (path: string, title: string) => void;
}

export function PostListSidebar({
  collections,
  activeCollection,
  setActiveCollection,
  posts,
  activePostPath,
  setActivePostPath,
  filterQuery,
  setFilterQuery,
  onCreatePost,
  onDeletePost,
}: PostListSidebarProps) {
  const filteredPosts = posts.filter((p) =>
    (p.title || p.slug).toLowerCase().includes(filterQuery.toLowerCase())
  );

  return (
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
          onClick={onCreatePost}
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
                  onDeletePost(post.path, post.title || post.slug);
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
  );
}
