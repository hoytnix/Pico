use crate::builder::post::BuiltPost;
use crate::models::SiteConfig;

fn xml_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn generate_sitemap_xml(base_url: &str, entries: &[(String, String)]) -> String {
    let base = base_url.trim_end_matches('/');
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");
    for (url, lastmod) in entries {
        let loc = if url.starts_with('/') {
            format!("{}{}", base, url)
        } else {
            format!("{}/{}", base, url)
        };
        xml.push_str("  <url>\n");
        xml.push_str(&format!("    <loc>{}</loc>\n", xml_escape(&loc)));
        if !lastmod.is_empty() {
            xml.push_str(&format!("    <lastmod>{}</lastmod>\n", xml_escape(lastmod)));
        }
        xml.push_str("  </url>\n");
    }
    xml.push_str("</urlset>\n");
    xml
}

pub fn generate_robots_txt(base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    format!("User-agent: *\nAllow: /\nSitemap: {}/sitemap.xml\n", base)
}

pub fn generate_rss_xml(site: &SiteConfig, posts: &[BuiltPost]) -> String {
    let base = site.base_url.trim_end_matches('/');
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<rss version=\"2.0\">\n  <channel>\n");
    xml.push_str(&format!("    <title>{}</title>\n", xml_escape(&site.title)));
    xml.push_str(&format!("    <link>{}/</link>\n", xml_escape(base)));
    xml.push_str(&format!(
        "    <description>{}</description>\n",
        xml_escape(&site.description)
    ));
    for post in posts {
        let link = format!("{}{}", base, post.url);
        xml.push_str("    <item>\n");
        xml.push_str(&format!("      <title>{}</title>\n", xml_escape(&post.title)));
        xml.push_str(&format!("      <link>{}</link>\n", xml_escape(&link)));
        xml.push_str(&format!("      <guid>{}</guid>\n", xml_escape(&link)));
        xml.push_str(&format!("      <pubDate>{}</pubDate>\n", xml_escape(&post.date)));
        xml.push_str(&format!(
            "      <description>{}</description>\n",
            xml_escape(&post.excerpt)
        ));
        xml.push_str("    </item>\n");
    }
    xml.push_str("  </channel>\n</rss>\n");
    xml
}

pub fn generate_atom_xml(site: &SiteConfig, posts: &[BuiltPost]) -> String {
    let base = site.base_url.trim_end_matches('/');
    let updated = posts
        .first()
        .map(|p| format!("{}T00:00:00Z", p.date))
        .unwrap_or_else(|| "2026-11-03T00:00:00Z".to_string());

    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<feed xmlns=\"http://www.w3.org/2005/Atom\">\n");
    xml.push_str(&format!("  <title>{}</title>\n", xml_escape(&site.title)));
    xml.push_str(&format!("  <link href=\"{}/\"/>\n", xml_escape(base)));
    xml.push_str(&format!("  <id>{}/</id>\n", xml_escape(base)));
    xml.push_str(&format!("  <updated>{}</updated>\n", xml_escape(&updated)));
    xml.push_str(&format!(
        "  <author><name>{}</name></author>\n",
        xml_escape(&site.author)
    ));
    for post in posts {
        let link = format!("{}{}", base, post.url);
        xml.push_str("  <entry>\n");
        xml.push_str(&format!("    <title>{}</title>\n", xml_escape(&post.title)));
        xml.push_str(&format!("    <link href=\"{}\"/>\n", xml_escape(&link)));
        xml.push_str(&format!("    <id>{}</id>\n", xml_escape(&link)));
        xml.push_str(&format!(
            "    <updated>{}T00:00:00Z</updated>\n",
            xml_escape(&post.date)
        ));
        xml.push_str(&format!(
            "    <summary>{}</summary>\n",
            xml_escape(&post.excerpt)
        ));
        xml.push_str("  </entry>\n");
    }
    xml.push_str("</feed>\n");
    xml
}
