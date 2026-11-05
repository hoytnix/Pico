    use crate::{
        create_site_impl, export_site_impl, revert_template_impl, save_post_impl,
        save_template_impl,
    };
    use crate::builder::{
        BuiltPost, build_site_pipeline, derive_image_variant_paths, generate_robots_txt,
        generate_rss_xml, generate_sitemap_xml, group_by_tag, paginate_chunks,
        render_markdown_to_html,
    };
    use crate::models::{ModelDef, ModelField, SiteConfig, VibeTarget};
    use crate::validation::{
        check_duplicate_slugs, parse_markdown_document, slugify, validate_frontmatter,
        validate_template_with_models,
    };
    use crate::vibe::{evaluate_vibe_proposal, load_vibe_site_snapshot};
    use serde_json::Value;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;

    fn sample_blog_model() -> ModelDef {
        ModelDef {
            name: "blog".to_string(),
            label: "Blog".to_string(),
            template: "blog.jinja".to_string(),
            list_template: "blog-list.jinja".to_string(),
            fields: vec![
                ModelField {
                    name: "title".to_string(),
                    label: "Title".to_string(),
                    field_type: "string".to_string(),
                    required: true,
                    default: None,
                },
                ModelField {
                    name: "date".to_string(),
                    label: "Date".to_string(),
                    field_type: "date".to_string(),
                    required: true,
                    default: None,
                },
                ModelField {
                    name: "tags".to_string(),
                    label: "Tags".to_string(),
                    field_type: "tags".to_string(),
                    required: false,
                    default: None,
                },
                ModelField {
                    name: "excerpt".to_string(),
                    label: "Excerpt".to_string(),
                    field_type: "string".to_string(),
                    required: false,
                    default: None,
                },
                ModelField {
                    name: "hero".to_string(),
                    label: "Hero image".to_string(),
                    field_type: "image".to_string(),
                    required: false,
                    default: None,
                },
                ModelField {
                    name: "draft".to_string(),
                    label: "Draft".to_string(),
                    field_type: "bool".to_string(),
                    required: false,
                    default: Some(Value::Bool(false)),
                },
                ModelField {
                    name: "slug".to_string(),
                    label: "Slug".to_string(),
                    field_type: "string".to_string(),
                    required: false,
                    default: None,
                },
            ],
        }
    }

    #[test]
    fn test_frontmatter_parse_and_model_validation() {
        let model = sample_blog_model();

        // 1. Valid post matching §4.3
        let valid_md = r#"---
title: "Hello World"
date: "2026-11-03"
tags: [pico, intro]
excerpt: "First post."
hero: "hero.webp"
draft: false
slug: "hello-world"
---

# Hello World

Body in Markdown, edited in Milkdown Crepe.
"#;
        let (fm, body, raw_yaml) =
            parse_markdown_document(valid_md, "content/blog/hello-world.md").unwrap();
        assert!(!body.contains("---"));
        assert!(body.starts_with("# Hello World"));
        assert!(
            validate_frontmatter(&fm, &model, "content/blog/hello-world.md", Some(&raw_yaml))
                .is_ok()
        );

        // 2. Missing required field (`title`)
        let missing_req_md = r#"---
date: "2026-11-03"
draft: false
---

Body without title.
"#;
        let (fm_missing, _, yaml_missing) =
            parse_markdown_document(missing_req_md, "content/blog/no-title.md").unwrap();
        let err_missing = validate_frontmatter(
            &fm_missing,
            &model,
            "content/blog/no-title.md",
            Some(&yaml_missing),
        )
        .unwrap_err();
        assert!(err_missing
            .iter()
            .any(|e| e.message.contains("Missing required field 'title'")));

        // 3. Unknown field (`author_bio` not in Blog model)
        let unknown_field_md = r#"---
title: "Post with Unknown Field"
date: "2026-11-03"
author_bio: "Not in model"
---

Body text.
"#;
        let (fm_unk, _, yaml_unk) =
            parse_markdown_document(unknown_field_md, "content/blog/unk.md").unwrap();
        let err_unk =
            validate_frontmatter(&fm_unk, &model, "content/blog/unk.md", Some(&yaml_unk))
                .unwrap_err();
        assert!(err_unk
            .iter()
            .any(|e| e.message.contains("Unknown field 'author_bio'")));
    }

    #[test]
    fn test_slugify_and_duplicate_slug_detection() {
        assert_eq!(slugify("Hello World!"), "hello-world");
        assert_eq!(
            slugify("  Pico --- Static   Site   Generator 2026!! "),
            "pico-static-site-generator-2026"
        );
        assert_eq!(slugify("already-slugged"), "already-slugged");

        let entries = vec![
            (
                "content/blog/first.md".to_string(),
                "hello-world".to_string(),
                2,
            ),
            (
                "content/blog/second.md".to_string(),
                "unique-post".to_string(),
                2,
            ),
            (
                "content/blog/duplicate.md".to_string(),
                "hello-world".to_string(),
                2,
            ),
        ];
        let dup_errors = check_duplicate_slugs(&entries);
        assert_eq!(dup_errors.len(), 1);
        assert_eq!(dup_errors[0].file, "content/blog/duplicate.md");
        assert!(dup_errors[0].message.contains("Duplicate slug 'hello-world'"));
    }

    #[test]
    fn test_pagination_chunking_0_1_10_11_25() {
        // 0 items
        let items_0: Vec<usize> = vec![];
        assert_eq!(paginate_chunks(&items_0, 10).len(), 0);

        // 1 item
        let items_1: Vec<usize> = (1..=1).collect();
        let chunks_1 = paginate_chunks(&items_1, 10);
        assert_eq!(chunks_1.len(), 1);
        assert_eq!(chunks_1[0].len(), 1);

        // 10 items
        let items_10: Vec<usize> = (1..=10).collect();
        let chunks_10 = paginate_chunks(&items_10, 10);
        assert_eq!(chunks_10.len(), 1);
        assert_eq!(chunks_10[0].len(), 10);

        // 11 items
        let items_11: Vec<usize> = (1..=11).collect();
        let chunks_11 = paginate_chunks(&items_11, 10);
        assert_eq!(chunks_11.len(), 2);
        assert_eq!(chunks_11[0].len(), 10);
        assert_eq!(chunks_11[1].len(), 1);

        // 25 items
        let items_25: Vec<usize> = (1..=25).collect();
        let chunks_25 = paginate_chunks(&items_25, 10);
        assert_eq!(chunks_25.len(), 3);
        assert_eq!(chunks_25[0].len(), 10);
        assert_eq!(chunks_25[1].len(), 10);
        assert_eq!(chunks_25[2].len(), 5);
    }

    #[test]
    fn test_tag_grouping() {
        #[derive(Clone)]
        struct TaggedItem {
            id: &'static str,
            tags: Vec<String>,
        }
        let items = vec![
            TaggedItem {
                id: "a",
                tags: vec!["rust".to_string(), "pico".to_string()],
            },
            TaggedItem {
                id: "b",
                tags: vec!["pico".to_string(), "web".to_string()],
            },
            TaggedItem {
                id: "c",
                tags: vec!["rust".to_string()],
            },
        ];
        let grouped = group_by_tag(&items, |i| &i.tags);
        assert_eq!(grouped.get("pico").unwrap().len(), 2);
        assert_eq!(grouped.get(" pico ").is_none(), true);
        assert_eq!(grouped.get("rust").unwrap()[0].id, "a");
        assert_eq!(grouped.get("web").unwrap().len(), 1);
    }

    #[test]
    fn test_all_four_starters_build_cleanly_on_first_open() {
        let tmp = TempDir::new().unwrap();
        for starter in ["blog", "portfolio", "landing", "docs"] {
            let dir = tmp.path().join(format!("site-{}", starter));
            let info = create_site_impl(&dir, starter, "Starter Test", "Tester").unwrap();
            assert_eq!(info.config.title, "Starter Test");
            assert_eq!(info.config.author, "Tester");

            let res = build_site_pipeline(&dir, false, false);
            assert!(
                res.success,
                "Starter '{}' failed to build cleanly: {:?}",
                starter, res.errors
            );
            assert!(dir.join("out/index.html").is_file());
            assert!(dir.join("out/sitemap.xml").is_file());
            assert!(dir.join("out/robots.txt").is_file());
            assert!(dir.join("out/feed.xml").is_file());
            assert!(dir.join("out/search.json").is_file());
            assert!(dir.join("out/404.html").is_file());
        }
    }

    #[tokio::test]
    async fn test_axum_preview_server_serves_built_site_and_live_reload() {
        use std::sync::Arc;
        use std::sync::atomic::AtomicU64;

        let tmp = TempDir::new().unwrap();
        let site_dir = tmp.path().join("preview-blog");
        create_site_impl(&site_dir, "blog", "Preview Site", "Author").unwrap();

        let build_res = build_site_pipeline(&site_dir, true, true);
        assert!(build_res.success);

        let rev = Arc::new(AtomicU64::new(42));
        let port = crate::preview::start_axum_preview_server(site_dir.join("out"), rev.clone())
            .await
            .unwrap();

        let client = reqwest::Client::new();
        let index_resp = client
            .get(format!("http://127.0.0.1:{}/", port))
            .send()
            .await
            .unwrap();
        assert!(index_resp.status().is_success());
        let index_html = index_resp.text().await.unwrap();
        assert!(index_html.contains("Preview Site"));
        assert!(index_html.contains("/__pico_livereload"));

        let lr_resp = client
            .get(format!("http://127.0.0.1:{}/__pico_livereload", port))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        assert_eq!(lr_resp, "42");
    }

    #[test]
    fn test_save_and_delete_model_impl() {
        let tmp = TempDir::new().unwrap();
        let site_dir = tmp.path().join("model-site");
        create_site_impl(&site_dir, "blog", "Model Site", "Tester").unwrap();

        let new_model = ModelDef {
            name: "event".to_string(),
            label: "Events".to_string(),
            template: "event.jinja".to_string(),
            list_template: "event-list.jinja".to_string(),
            fields: vec![
                ModelField {
                    name: "title".to_string(),
                    label: "Title".to_string(),
                    field_type: "string".to_string(),
                    required: true,
                    default: None,
                },
                ModelField {
                    name: "date".to_string(),
                    label: "Event Date".to_string(),
                    field_type: "date".to_string(),
                    required: true,
                    default: None,
                },
                ModelField {
                    name: "location".to_string(),
                    label: "Location".to_string(),
                    field_type: "string".to_string(),
                    required: false,
                    default: None,
                },
            ],
        };

        crate::save_model_impl(&site_dir, &new_model).unwrap();
        let models_after_save = crate::builder::load_models(&site_dir).unwrap();
        assert!(models_after_save.iter().any(|m| m.name == "event"));

        // Delete model
        crate::delete_model_impl(&site_dir, "event").unwrap();
        let models_after_del = crate::builder::load_models(&site_dir).unwrap();
        assert!(!models_after_del.iter().any(|m| m.name == "event"));
    }

    #[test]
    fn test_embedded_starters_fallback_extracts_all_starters() {
        let tmp = TempDir::new().unwrap();
        for starter in ["blog", "portfolio", "landing", "docs"] {
            let dir = tmp.path().join(format!("embedded-{}", starter));
            let embedded = crate::EMBEDDED_STARTERS.get_dir(starter).unwrap();
            fs::create_dir_all(&dir).unwrap();
            crate::extract_embedded_starter(embedded, &dir, starter).unwrap();
            assert!(dir.join("site.yaml").is_file());
            assert!(dir.join("templates/skeleton.jinja").is_file());
            assert!(dir.join("models").is_dir());
            assert!(dir.join("content").is_dir());
        }
    }

    #[test]
    fn test_image_resize_path_derivation() {
        assert_eq!(
            derive_image_variant_paths("hero.png"),
            Some(("hero-hero.webp".to_string(), "hero-thumb.webp".to_string()))
        );
        assert_eq!(
            derive_image_variant_paths("assets/photo.JPG"),
            Some((
                "assets/photo-hero.webp".to_string(),
                "assets/photo-thumb.webp".to_string()
            ))
        );
        // SVG, GIF, AVIF are unchanged in MVP
        assert_eq!(derive_image_variant_paths("logo.svg"), None);
        assert_eq!(derive_image_variant_paths("anim.gif"), None);
        assert_eq!(derive_image_variant_paths("banner.avif"), None);
    }

    #[test]
    fn test_sitemap_robots_and_rss_generation() {
        let entries = vec![
            ("/".to_string(), "2026-11-03".to_string()),
            ("/blog/hello-world/".to_string(), "2026-11-03".to_string()),
        ];
        let sitemap = generate_sitemap_xml("https://example.com", &entries);
        assert!(sitemap.contains("<loc>https://example.com/</loc>"));
        assert!(sitemap.contains("<loc>https://example.com/blog/hello-world/</loc>"));
        assert!(sitemap.contains("<lastmod>2026-11-03</lastmod>"));

        let robots = generate_robots_txt("https://example.com");
        assert!(robots.contains("Allow: /"));
        assert!(robots.contains("Sitemap: https://example.com/sitemap.xml"));

        let site = SiteConfig::default();
        let posts = vec![BuiltPost {
            title: "Hello World".to_string(),
            date: "2026-11-03".to_string(),
            slug: "hello-world".to_string(),
            draft: false,
            excerpt: "First post.".to_string(),
            hero: String::new(),
            hero_webp: String::new(),
            thumb_webp: String::new(),
            tags: vec!["pico".to_string()],
            url: "/blog/hello-world/".to_string(),
            collection: "blog".to_string(),
            body: String::new(),
            body_html: "<p>Hello</p>".to_string(),
            body_text: "Hello".to_string(),
            attachments: vec![],
            source_rel_path: "content/blog/hello-world.md".to_string(),
            markdown_images: vec![],
            extra_fields: serde_json::Map::new(),
        }];
        let rss = generate_rss_xml(&site, &posts);
        assert!(rss.contains("<rss version=\"2.0\">"));
        assert!(rss.contains("<title>Hello World</title>"));
        assert!(rss.contains("<link>https://example.com/blog/hello-world/</link>"));
    }

    #[test]
    fn test_minijinja_macro_import_and_unknown_variable_rejection() {
        let model = sample_blog_model();
        let mut templates = HashMap::new();
        templates.insert(
            "skeleton.jinja".to_string(),
            "<!DOCTYPE html><html><head><title>{% block title %}{{ site.title }}{% endblock %}</title></head><body>{% block content %}{% endblock %}</body></html>".to_string(),
        );
        templates.insert(
            "macros/cards.jinja".to_string(),
            "{% macro post_card(post) %}\n<article><h2><a href=\"{{ post.url }}\">{{ post.title }}</a></h2><p>{{ post.excerpt }}</p></article>\n{% endmacro %}\n".to_string(),
        );

        // 1. Valid macro import in blog-list.jinja
        let valid_list = "{% extends \"skeleton.jinja\" %}\n{% from \"macros/cards.jinja\" import post_card %}\n{% block content %}{% for post in posts %}{{ post_card(post) }}{% endfor %}{% endblock %}\n";
        assert!(validate_template_with_models(
            "blog-list.jinja",
            valid_list,
            &templates,
            &[model.clone()]
        )
        .is_ok());

        // 2. Unknown variable in blog.jinja (`post.author_bio`) must be refused with friendly message naming file, line, and variable (§3.4)
        let broken_blog = "{% extends \"skeleton.jinja\" %}\n{% block content %}\n<h1>{{ post.title }}</h1>\n<p>{{ post.author_bio }}</p>\n{% endblock %}\n";
        let err = validate_template_with_models(
            "blog.jinja",
            broken_blog,
            &templates,
            &[model.clone()],
        )
        .unwrap_err();
        assert_eq!(err.file, "blog.jinja");
        assert_eq!(err.line, 4);
        assert!(
            err.message
                .contains("blog.jinja uses `post.author_bio`, but Blog posts don't have `author_bio`"),
            "Unexpected error message: {}",
            err.message
        );

        // 3. Macro file with top-level HTML outside {% macro %} must be rejected (§4.5)
        let bad_macro_html = "<div>Not allowed outside macro</div>\n{% macro post_card(post) %}<p>{{ post.title }}</p>{% endmacro %}\n";
        let err_macro = validate_template_with_models(
            "macros/cards.jinja",
            bad_macro_html,
            &templates,
            &[model.clone()],
        )
        .unwrap_err();
        assert!(err_macro.message.contains("ONLY `{% macro %}`"));

        // 4. Macro using kwargs must be rejected (§4.6)
        let bad_kwargs = "{% macro post_card(post, **kwargs) %}<p>{{ post.title }}</p>{% endmacro %}\n";
        let err_kw = validate_template_with_models(
            "macros/cards.jinja",
            bad_kwargs,
            &templates,
            &[model],
        )
        .unwrap_err();
        assert!(err_kw.message.contains("kwargs"));
    }

    #[test]
    fn test_end_to_end_starter_lifecycle_pagination_drafts_revert_export_and_vibe() {
        let tmp = TempDir::new().unwrap();
        let site_dir = tmp.path().join("my-blog");

        // 1. Create fresh Blog starter site
        let info = create_site_impl(&site_dir, "blog", "Lumina Journal", "Ada Lovelace").unwrap();
        assert_eq!(info.config.title, "Lumina Journal");
        assert_eq!(info.config.author, "Ada Lovelace");

        // 2. Add a post in Markdown with heading, bold, list, fenced rust code block, and image
        let (html, _, imgs) = render_markdown_to_html(
            "# Deep Dive\n\n**Bold insight**\n\n- Item 1\n- Item 2\n\n```rust\nfn demo() {}\n```\n\n![Hero](hero.png)\n",
        );
        assert!(html.contains("<strong>Bold insight</strong>"));
        assert!(html.contains("language-rust"));
        assert_eq!(imgs, vec!["hero.png".to_string()]);

        // 3. Edit `templates/macros/cards.jinja` via save_template_impl, then test broken template rejection & Revert
        let mut backups = HashMap::new();
        let updated_cards = "{% macro post_card(post) %}\n<article class=\"post-card custom-card\"><h2><a href=\"{{ post.url }}\">{{ post.title }}</a></h2><span class=\"card-date\">{{ post.date }}</span><p>{{ post.excerpt }}</p></article>\n{% endmacro %}\n";
        save_template_impl(&site_dir, "macros/cards.jinja", updated_cards, &mut backups).unwrap();

        // Break `blog.jinja` on purpose with `post.author_bio` -> save is refused, previous file intact
        let orig_blog_tpl = fs::read_to_string(site_dir.join("templates/blog.jinja")).unwrap();
        let bad_blog_tpl = format!("{}\n{{{{ post.author_bio }}}}", orig_blog_tpl);
        let save_err =
            save_template_impl(&site_dir, "blog.jinja", &bad_blog_tpl, &mut backups).unwrap_err();
        assert!(save_err.contains("post.author_bio"));
        assert_eq!(
            fs::read_to_string(site_dir.join("templates/blog.jinja")).unwrap(),
            orig_blog_tpl
        );

        // Revert `macros/cards.jinja` and re-apply custom card
        let reverted = revert_template_impl(&site_dir, "macros/cards.jinja", &mut backups).unwrap();
        assert!(!reverted.contains("custom-card"));
        save_template_impl(&site_dir, "macros/cards.jinja", updated_cards, &mut backups).unwrap();

        // 4. Add posts so total published posts >= 12 (starter has 3, add 10 published + 1 draft)
        for i in 1..=10 {
            let slug = format!("extra-post-{}", i);
            let fm = serde_json::json!({
                "title": format!("Extra Post {}", i),
                "date": format!("2026-10-{:02}", i),
                "tags": ["pico", "guide"],
                "excerpt": format!("Summary for post {}", i),
                "hero": "",
                "draft": false,
                "slug": slug
            });
            let path = format!("content/blog/{}.md", slug);
            save_post_impl(
                &site_dir,
                &path,
                fm,
                &format!("# Extra Post {}\n\nContent for post {}.", i, i),
            )
            .unwrap();
        }

        // Add 1 draft post
        let draft_fm = serde_json::json!({
            "title": "Secret Draft Post",
            "date": "2026-11-04",
            "tags": ["drafts"],
            "excerpt": "Work in progress",
            "hero": "",
            "draft": true,
            "slug": "secret-draft-post"
        });
        save_post_impl(
            &site_dir,
            "content/blog/secret-draft-post.md",
            draft_fm,
            "# Secret Draft\n\nNot ready yet.",
        )
        .unwrap();

        // 5. Preview build (`include_drafts = true`) includes draft with visible DRAFT badge
        let preview_res = build_site_pipeline(&site_dir, true, true);
        assert!(preview_res.success, "Preview build errors: {:?}", preview_res.errors);
        let draft_html =
            fs::read_to_string(site_dir.join("out/blog/secret-draft-post/index.html")).unwrap();
        assert!(draft_html.contains("DRAFT"));

        // 6. Final Build (`include_drafts = false`) excludes draft, paginates 13 published posts into 2 pages,
        // generates WebP variants, sitemap.xml, robots.txt, feed.xml, search.json, 404.html
        let build_res = build_site_pipeline(&site_dir, false, false);
        assert!(build_res.success, "Build errors: {:?}", build_res.errors);
        assert!(!site_dir.join("out/blog/secret-draft-post/index.html").exists());
        assert!(site_dir.join("out/index.html").is_file());
        assert!(site_dir.join("out/blog/index.html").is_file());
        assert!(site_dir.join("out/blog/page/2/index.html").is_file());
        assert!(site_dir.join("out/tags/pico/index.html").is_file());
        assert!(site_dir.join("out/sitemap.xml").is_file());
        assert!(site_dir.join("out/robots.txt").is_file());
        assert!(site_dir.join("out/feed.xml").is_file());
        assert!(site_dir.join("out/search.json").is_file());
        assert!(site_dir.join("out/404.html").is_file());
        assert!(site_dir
            .join("out/blog/hello-world/hero-hero.webp")
            .is_file());
        assert!(site_dir
            .join("out/blog/hello-world/hero-thumb.webp")
            .is_file());

        // 7. Export copies `out/` to a picked folder
        let export_dir = tmp.path().join("exported-site");
        export_site_impl(&site_dir, &export_dir).unwrap();
        assert!(export_dir.join("index.html").is_file());
        assert!(export_dir.join("blog/page/2/index.html").is_file());
        assert!(export_dir.join("search.json").is_file());

        // 8. Simulate Vibe diff + validation with mock LLM output (in tests only, §11.4)
        let vibe_target = VibeTarget {
            kind: "template".to_string(),
            path: "macros/cards.jinja".to_string(),
        };
        let valid_vibe_proposal = "{% macro post_card(post) %}\n<article class=\"post-card vibe-updated\"><h2><a href=\"{{ post.url }}\">{{ post.title }}</a></h2><time>{{ post.date }}</time></article>\n{% endmacro %}\n";
        let vibe_ok = evaluate_vibe_proposal(
            &site_dir,
            &vibe_target,
            updated_cards,
            valid_vibe_proposal,
        )
        .unwrap();
        assert!(vibe_ok.validation_error.is_none());
        assert!(vibe_ok.diff.contains("+<article class=\"post-card vibe-updated\">"));

        // Invalid LLM proposal using unknown variable is flagged by validation guardrail
        let invalid_vibe_proposal = "{% macro post_card(post) %}\n<article>{{ post.nonexistent_var }}</article>\n{% endmacro %}\n";
        let vibe_bad = evaluate_vibe_proposal(
            &site_dir,
            &vibe_target,
            updated_cards,
            invalid_vibe_proposal,
        )
        .unwrap();
        assert!(vibe_bad.validation_error.is_some());
        assert!(vibe_bad
            .validation_error
            .unwrap()
            .contains("post.nonexistent_var"));

        // 9. Simulate Vibe model editing diff + validation
        let model_target = VibeTarget {
            kind: "model".to_string(),
            path: "models/blog.yaml".to_string(),
        };
        let original_model_yaml = std::fs::read_to_string(site_dir.join("models").join("blog.yaml")).unwrap();
        let valid_model_proposal = "name: blog\nlabel: Blog Articles\ntemplate: blog.jinja\nlist_template: blog-list.jinja\nfields:\n  - name: title\n    label: Title\n    type: string\n    required: true\n  - name: date\n    label: Date\n    type: date\n    required: true\n  - name: rating\n    label: Rating\n    type: string\n    required: false\n";
        let model_vibe_ok = evaluate_vibe_proposal(
            &site_dir,
            &model_target,
            &original_model_yaml,
            valid_model_proposal,
        )
        .unwrap();
        assert!(model_vibe_ok.validation_error.is_none());
        assert!(model_vibe_ok.diff.contains("+    label: Rating"));

        // Invalid model proposal (invalid YAML / invalid field type)
        let invalid_model_proposal = "name: blog\nlabel: Blog\ntemplate: blog.jinja\nlist_template: blog-list.jinja\nfields:\n  - name: bad_field\n    label: Bad\n    type: unsupported_type_xyz\n";
        let model_vibe_bad = evaluate_vibe_proposal(
            &site_dir,
            &model_target,
            &original_model_yaml,
            invalid_model_proposal,
        )
        .unwrap();
        assert!(model_vibe_bad.validation_error.is_some());
        assert!(model_vibe_bad
            .validation_error
            .unwrap()
            .contains("unsupported type 'unsupported_type_xyz'"));
    }

    #[test]
    fn test_load_vibe_site_snapshot() {
        let tmp = TempDir::new().unwrap();
        let site_dir = tmp.path().join("vibe-snapshot-site");
        create_site_impl(&site_dir, "blog", "Snapshot Test", "Snapshot Author").unwrap();

        let snapshot = load_vibe_site_snapshot(&site_dir).expect("load_vibe_site_snapshot failed");

        // Verify site.yaml loaded
        assert!(snapshot.site_yaml.contains("Snapshot Test"));
        assert!(snapshot.site_yaml.contains("Snapshot Author"));

        // Verify models loaded (blog.yaml and project.yaml)
        assert!(snapshot.models.iter().any(|m| m.path == "models/blog.yaml" && m.content.contains("name: blog")));
        assert!(snapshot.models.iter().any(|m| m.path == "models/project.yaml" && m.content.contains("name: project")));

        // Verify theme CSS loaded
        assert!(!snapshot.theme_css.is_empty());
        assert!(snapshot.theme_css.contains("--bg"));

        // Verify templates loaded (skeleton, blog, project, 404, macros/cards)
        assert!(snapshot.templates.iter().any(|t| t.path == "templates/skeleton.jinja"));
        assert!(snapshot.templates.iter().any(|t| t.path == "templates/blog.jinja"));
        assert!(snapshot.templates.iter().any(|t| t.path == "templates/macros/cards.jinja"));

        // Verify content manifest loaded
        assert!(!snapshot.content_manifest.is_empty());
        assert!(snapshot.content_manifest.iter().any(|c| c.slug == "hello-world" && c.collection == "blog"));
    }
