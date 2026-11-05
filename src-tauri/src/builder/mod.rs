pub mod assets;
pub mod content_scanner;
pub mod feeds;
pub mod models_loader;
pub mod pagination;
pub mod pipeline;
pub mod post;
pub mod render;

// Re-exports for complete backward compatibility across pico_lib and tests
pub use assets::{copy_dir_recursive, generate_webp_variants_for_file};
pub use feeds::{generate_atom_xml, generate_robots_txt, generate_rss_xml, generate_sitemap_xml};
pub use models_loader::{
    load_all_templates, load_models, load_site_config, resolve_collection_folder,
};
pub use pagination::{group_by_tag, paginate_chunks};
pub use pipeline::build_site_pipeline;
pub use post::{derive_image_variant_paths, render_markdown_to_html, BuiltPost};
pub use render::maybe_inject_live_reload;
