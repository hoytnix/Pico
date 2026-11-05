use crate::builder::assets::{copy_dir_recursive, generate_webp_variants_for_file};
use crate::builder::content_scanner::scan_collection_content;
use crate::builder::feeds::{
    generate_atom_xml, generate_robots_txt, generate_rss_xml, generate_sitemap_xml,
};
use crate::builder::models_loader::{
    load_all_templates, load_models, load_site_config, resolve_collection_folder,
};
use crate::builder::post::BuiltPost;
use crate::builder::render::render_site_templates;
use crate::models::{BuildError, BuildResult};
use crate::validation::validate_template_with_models;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn build_site_pipeline(
    site_root: &Path,
    include_drafts: bool,
    inject_live_reload: bool,
) -> BuildResult {
    log::info!(
        target: "pico::builder",
        "Build pipeline started: site_root={:?}, include_drafts={}, inject_live_reload={}",
        site_root,
        include_drafts,
        inject_live_reload
    );
    let out_final = site_root.join("out");
    let out_tmp = site_root.join("out.tmp");
    let mut errors: Vec<BuildError> = Vec::new();
    let mut log: Vec<String> = Vec::new();
    let mut images_generated = 0usize;

    // Clean any stale out.tmp/
    if out_tmp.exists() {
        let _ = fs::remove_dir_all(&out_tmp);
    }
    if let Err(e) = fs::create_dir_all(&out_tmp) {
        log::error!(target: "pico::builder", "Could not create temporary out.tmp directory: {}", e);
        return BuildResult {
            success: false,
            out_dir: out_final.to_string_lossy().to_string(),
            errors: vec![BuildError {
                file: "out.tmp".to_string(),
                line: 0,
                message: format!("Could not create temporary build directory: {}", e),
            }],
            pages_built: 0,
            images_generated: 0,
            log: vec![format!("Failed to create out.tmp: {}", e)],
        };
    }

    let site_config = match load_site_config(site_root) {
        Ok(cfg) => cfg,
        Err(err) => {
            log::error!(target: "pico::builder", "Failed to load site.yaml: {}", err);
            let _ = fs::remove_dir_all(&out_tmp);
            return BuildResult {
                success: false,
                out_dir: out_final.to_string_lossy().to_string(),
                errors: vec![err],
                pages_built: 0,
                images_generated: 0,
                log: vec!["Failed to load site.yaml".to_string()],
            };
        }
    };

    let models = match load_models(site_root) {
        Ok(m) => m,
        Err(err) => {
            let _ = fs::remove_dir_all(&out_tmp);
            return BuildResult {
                success: false,
                out_dir: out_final.to_string_lossy().to_string(),
                errors: vec![err],
                pages_built: 0,
                images_generated: 0,
                log: vec!["Failed to load models/".to_string()],
            };
        }
    };

    let templates = match load_all_templates(site_root) {
        Ok(t) => t,
        Err(err) => {
            let _ = fs::remove_dir_all(&out_tmp);
            return BuildResult {
                success: false,
                out_dir: out_final.to_string_lossy().to_string(),
                errors: vec![err],
                pages_built: 0,
                images_generated: 0,
                log: vec!["Failed to load templates/".to_string()],
            };
        }
    };

    // Validate all templates upfront
    for (tpl_name, tpl_src) in &templates {
        if let Err(err) = validate_template_with_models(tpl_name, tpl_src, &templates, &models) {
            errors.push(err);
        }
    }

    // Step 1: Scan and parse collection markdown content
    let scanned = scan_collection_content(site_root, &models, include_drafts);
    let collection_posts = scanned.collection_posts;
    let collection_folders = scanned.collection_folders;
    errors.extend(scanned.errors);
    log.extend(scanned.log);

    // Step 2: Render MiniJinja pages
    let rendered = render_site_templates(
        &out_tmp,
        &site_config,
        &models,
        &templates,
        &collection_posts,
        &collection_folders,
        include_drafts,
        inject_live_reload,
    );
    let pages_built = rendered.pages_built;
    let sitemap_entries = rendered.sitemap_entries;
    let search_entries = rendered.search_entries;
    errors.extend(rendered.errors);

    // Step 3: Copy assets and page attachments
    let assets_src = site_root.join("assets");
    let assets_dst = out_tmp.join("assets");
    if assets_src.is_dir() {
        if let Err(e) = copy_dir_recursive(&assets_src, &assets_dst) {
            errors.push(BuildError {
                file: "assets/".to_string(),
                line: 0,
                message: format!("Failed to copy assets/: {}", e),
            });
        }
    }

    for model in &models {
        let (_, col_dir) = resolve_collection_folder(site_root, &model.name);
        if let Some(posts) = collection_posts.get(&model.name) {
            for post in posts {
                let attach_src = col_dir.join(&post.slug);
                if attach_src.is_dir() {
                    let attach_dst = out_tmp.join(&post.collection).join(&post.slug);
                    if let Err(e) = copy_dir_recursive(&attach_src, &attach_dst) {
                        errors.push(BuildError {
                            file: post.source_rel_path.clone(),
                            line: 1,
                            message: format!("Failed to copy page attachments: {}", e),
                        });
                    }
                }
            }
        }
    }

    // Step 4: WebP Image Variant Generation
    let mut processed_images: HashSet<PathBuf> = HashSet::new();

    for model in &models {
        let (_, col_dir) = resolve_collection_folder(site_root, &model.name);
        if let Some(posts) = collection_posts.get(&model.name) {
            for post in posts {
                let mut refs: Vec<String> = Vec::new();
                if !post.hero.is_empty() {
                    refs.push(post.hero.clone());
                }
                for md_img in &post.markdown_images {
                    refs.push(md_img.clone());
                }

                for img_ref in refs {
                    if img_ref.starts_with("http://") || img_ref.starts_with("https://") {
                        continue;
                    }
                    let clean_ref = img_ref.trim_start_matches("./").trim_start_matches('/');
                    let ext = Path::new(clean_ref)
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|s| s.to_ascii_lowercase())
                        .unwrap_or_default();

                    if !matches!(ext.as_str(), "png" | "jpg" | "jpeg") {
                        continue;
                    }

                    let cand1 = col_dir.join(&post.slug).join(clean_ref);
                    let cand2 = col_dir.join(clean_ref);
                    let cand3 = site_root.join(clean_ref);
                    let cand4 = assets_src.join(clean_ref);

                    let (src_img_path, out_img_dir) = if cand1.is_file() {
                        (cand1, out_tmp.join(&post.collection).join(&post.slug))
                    } else if cand2.is_file() {
                        (cand2, out_tmp.join(&post.collection).join(&post.slug))
                    } else if cand3.is_file() {
                        let parent_rel = Path::new(clean_ref).parent().unwrap_or(Path::new(""));
                        (cand3, out_tmp.join(parent_rel))
                    } else if cand4.is_file() {
                        (cand4, out_tmp.join("assets"))
                    } else {
                        errors.push(BuildError {
                            file: post.source_rel_path.clone(),
                            line: 1,
                            message: format!(
                                "Referenced image '{}' was not found on disk.",
                                img_ref
                            ),
                        });
                        continue;
                    };

                    if processed_images.insert(src_img_path.clone()) {
                        let rel_disp = src_img_path
                            .strip_prefix(site_root)
                            .unwrap_or(&src_img_path)
                            .to_string_lossy()
                            .replace('\\', "/");
                        match generate_webp_variants_for_file(
                            &src_img_path,
                            &out_img_dir,
                            &rel_disp,
                        ) {
                            Ok(count) => {
                                images_generated += count;
                                if let Some(fname) = src_img_path.file_name() {
                                    let dest_orig = out_img_dir.join(fname);
                                    if !dest_orig.exists() {
                                        let _ = fs::copy(&src_img_path, &dest_orig);
                                    }
                                }
                            }
                            Err(err) => errors.push(err),
                        }
                    }
                }
            }
        }
    }

    if assets_src.is_dir() {
        for entry in WalkDir::new(&assets_src).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let p = entry.path().to_path_buf();
                let ext = p
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_ascii_lowercase())
                    .unwrap_or_default();
                if matches!(ext.as_str(), "png" | "jpg" | "jpeg") && processed_images.insert(p.clone()) {
                    let rel_disp = p
                        .strip_prefix(site_root)
                        .unwrap_or(&p)
                        .to_string_lossy()
                        .replace('\\', "/");
                    let rel_from_assets = p
                        .strip_prefix(&assets_src)
                        .unwrap_or(&p)
                        .parent()
                        .unwrap_or(Path::new(""));
                    let target_dir = assets_dst.join(rel_from_assets);
                    match generate_webp_variants_for_file(&p, &target_dir, &rel_disp) {
                        Ok(count) => images_generated += count,
                        Err(err) => errors.push(err),
                    }
                }
            }
        }
    }

    // Step 5: Feeds & Search Index
    let sitemap_xml = generate_sitemap_xml(&site_config.base_url, &sitemap_entries);
    let _ = fs::write(out_tmp.join("sitemap.xml"), sitemap_xml);

    let robots_txt = generate_robots_txt(&site_config.base_url);
    let _ = fs::write(out_tmp.join("robots.txt"), robots_txt);

    let blog_posts: Vec<BuiltPost> = collection_posts
        .get("blog")
        .cloned()
        .unwrap_or_else(|| {
            models
                .first()
                .and_then(|m| collection_posts.get(&m.name))
                .cloned()
                .unwrap_or_default()
        });
    let rss_xml = generate_rss_xml(&site_config, &blog_posts);
    let _ = fs::write(out_tmp.join("feed.xml"), rss_xml);

    let atom_xml = generate_atom_xml(&site_config, &blog_posts);
    let _ = fs::write(out_tmp.join("atom.xml"), atom_xml);

    let search_json =
        serde_json::to_string_pretty(&search_entries).unwrap_or_else(|_| "[]".to_string());
    let _ = fs::write(out_tmp.join("search.json"), search_json);

    // Step 6: Atomic directory swap
    log.push(format!(
        "Pages built: {}, WebP image variants generated: {}, Errors: {}",
        pages_built,
        images_generated,
        errors.len()
    ));
    for err in &errors {
        log.push(format!("ERROR: {}", err));
    }

    if !errors.is_empty() {
        log::error!(
            target: "pico::builder",
            "Build failed with {} errors: {:?}",
            errors.len(),
            errors
        );
        let _ = fs::remove_dir_all(&out_tmp);
        return BuildResult {
            success: false,
            out_dir: out_final.to_string_lossy().to_string(),
            errors,
            pages_built: 0,
            images_generated: 0,
            log,
        };
    }

    let _ = fs::write(out_tmp.join("build.log"), log.join("\n"));

    if out_final.exists() {
        let _ = fs::remove_dir_all(&out_final);
    }
    if let Err(e) = fs::rename(&out_tmp, &out_final) {
        if let Err(copy_err) = copy_dir_recursive(&out_tmp, &out_final) {
            log::error!(
                target: "pico::builder",
                "Failed to finalize out/ directory (rename err: {}, copy err: {})",
                e,
                copy_err
            );
            return BuildResult {
                success: false,
                out_dir: out_final.to_string_lossy().to_string(),
                errors: vec![BuildError {
                    file: "out/".to_string(),
                    line: 0,
                    message: format!("Failed to finalize out/ directory ({}): {}", e, copy_err),
                }],
                pages_built: 0,
                images_generated: 0,
                log,
            };
        }
        let _ = fs::remove_dir_all(&out_tmp);
    }

    log::info!(
        target: "pico::builder",
        "Build finished successfully: pages_built={}, images_generated={}, out_dir={:?}",
        pages_built,
        images_generated,
        out_final
    );

    BuildResult {
        success: true,
        out_dir: out_final.to_string_lossy().to_string(),
        errors: Vec::new(),
        pages_built,
        images_generated,
        log,
    }
}
