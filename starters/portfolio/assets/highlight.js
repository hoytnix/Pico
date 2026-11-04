/* Pico Self-Contained Offline Syntax Highlighter (highlight.js compatible API) */
(function () {
  function escapeHtml(str) {
    return str
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
  }

  function highlightTokenized(code) {
    var tokens = [];
    function stash(cls, text) {
      var id = "\u0000" + tokens.length + "\u0000";
      tokens.push('<span class="hljs-' + cls + '">' + escapeHtml(text) + "</span>");
      return id;
    }

    var working = code;
    // Comments
    working = working.replace(/(\/\/[^\n]*|#[^\n]*|\/\*[\s\S]*?\*\/)/g, function (m) {
      return stash("comment", m);
    });
    // Strings
    working = working.replace(/("(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|`(?:\\.|[^`\\])*`)/g, function (m) {
      return stash("string", m);
    });
    // Escape remaining HTML before keyword/number matching
    working = escapeHtml(working);
    // Keywords
    working = working.replace(
      /\b(fn|let|mut|pub|struct|enum|impl|trait|use|mod|match|if|else|for|while|loop|return|async|await|const|var|function|class|interface|type|import|export|from|def|True|False|None|true|false|null|Self|self|where|in|as)\b/g,
      '<span class="hljs-keyword">$1</span>'
    );
    // Types / Builtins
    working = working.replace(
      /\b(String|Vec|Option|Result|Some|None|Ok|Err|u8|u16|u32|u64|usize|i8|i16|i32|i64|isize|f32|f64|bool|str|Promise|Record|Array|Map|Set|console|window|document)\b/g,
      '<span class="hljs-type">$1</span>'
    );
    // Numbers
    working = working.replace(/\b(\d+(?:\.\d+)?)\b/g, '<span class="hljs-number">$1</span>');

    // Restore stashed tokens
    working = working.replace(/\u0000(\d+)\u0000/g, function (_, idx) {
      return tokens[Number(idx)];
    });
    return working;
  }

  function highlightElement(block) {
    if (block.dataset.highlighted === "yes") return;
    var cls = block.className || "";
    var langMatch = cls.match(/language-([a-zA-Z0-9_-]+)/);
    var lang = langMatch ? langMatch[1] : "";
    var raw = block.textContent || "";
    var badge = lang ? '<span class="hljs-lang-badge">' + escapeHtml(lang) + "</span>" : "";
    block.classList.add("hljs");
    block.innerHTML = badge + highlightTokenized(raw);
    block.dataset.highlighted = "yes";
  }

  function highlightAll() {
    var blocks = document.querySelectorAll("pre code");
    blocks.forEach(highlightElement);
  }

  window.hljs = {
    highlightElement: highlightElement,
    highlightAll: highlightAll
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", highlightAll);
  } else {
    highlightAll();
  }
})();
