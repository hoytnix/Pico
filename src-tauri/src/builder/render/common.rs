use crate::models::BuildError;
use serde_json::Value;
use std::fs;
use std::path::Path;

pub struct RenderedPagesResult {
    pub pages_built: usize,
    pub sitemap_entries: Vec<(String, String)>,
    pub search_entries: Vec<Value>,
    pub errors: Vec<BuildError>,
}

pub fn maybe_inject_live_reload(html: String, inject: bool) -> String {
    if !inject {
        return html;
    }
    let script = r#"<script>
(function() {
  var rev = null;
  function check() {
    fetch('/__pico_livereload')
      .then(function(r) { return r.text(); })
      .then(function(txt) {
        var r = parseInt(txt, 10);
        if (rev === null) {
          rev = r;
        } else if (r > rev) {
          window.location.reload();
        }
      })
      .catch(function() {});
  }
  setInterval(check, 400);
})();
</script>"#;

    if let Some(pos) = html.to_lowercase().rfind("</body>") {
        let mut out = html;
        out.insert_str(pos, script);
        out
    } else {
        format!("{}{}", html, script)
    }
}

/// Atomically writes an HTML page to target directory, recording file system errors.
pub fn write_atomic_page(
    target_dir: &Path,
    html: &str,
    source_file: &str,
    errors: &mut Vec<BuildError>,
) -> bool {
    if let Err(e) = fs::create_dir_all(target_dir) {
        errors.push(BuildError {
            file: source_file.to_string(),
            line: 1,
            message: format!("Failed to create output directory: {}", e),
        });
        return false;
    }
    if let Err(e) = fs::write(target_dir.join("index.html"), html) {
        errors.push(BuildError {
            file: source_file.to_string(),
            line: 1,
            message: format!("Failed to write index.html: {}", e),
        });
        return false;
    }
    true
}

/// Calculates previous and next page URLs for paginated views.
pub fn calculate_pagination_urls(
    base_folder: &str,
    page_num: usize,
    total_pages: usize,
) -> (String, String) {
    let prev_url = if page_num <= 1 {
        String::new()
    } else if page_num == 2 {
        format!("/{}/", base_folder)
    } else {
        format!("/{}/page/{}/", base_folder, page_num - 1)
    };
    let next_url = if page_num < total_pages {
        format!("/{}/page/{}/", base_folder, page_num + 1)
    } else {
        String::new()
    };
    (prev_url, next_url)
}
