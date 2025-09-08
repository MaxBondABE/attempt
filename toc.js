// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="intro.html">Introduction</a></li><li class="chapter-item expanded affix "><li class="part-title">Getting started</li><li class="chapter-item expanded "><a href="usage.html"><strong aria-hidden="true">1.</strong> Example usage</a></li><li class="chapter-item expanded "><a href="install.html"><strong aria-hidden="true">2.</strong> Installation</a></li><li class="chapter-item expanded affix "><li class="part-title">Usage</li><li class="chapter-item expanded "><a href="backoff/intro.html"><strong aria-hidden="true">3.</strong> Backoff schedule</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="backoff/fixed.html"><strong aria-hidden="true">3.1.</strong> Fixed delay</a></li><li class="chapter-item expanded "><a href="backoff/exponential.html"><strong aria-hidden="true">3.2.</strong> Exponential backoff</a></li><li class="chapter-item expanded "><a href="backoff/linear.html"><strong aria-hidden="true">3.3.</strong> Linear backoff</a></li></ol></li><li class="chapter-item expanded "><a href="policy/intro.html"><strong aria-hidden="true">4.</strong> Policy controls</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="policy/default.html"><strong aria-hidden="true">4.1.</strong> The default policy</a></li><li class="chapter-item expanded "><a href="policy/retrying.html"><strong aria-hidden="true">4.2.</strong> Retry controls</a></li><li class="chapter-item expanded "><a href="policy/status.html"><strong aria-hidden="true">4.3.</strong> Status predicates</a></li><li class="chapter-item expanded "><a href="policy/output.html"><strong aria-hidden="true">4.4.</strong> Output predicates</a></li><li class="chapter-item expanded "><a href="policy/signal.html"><strong aria-hidden="true">4.5.</strong> Timeout &amp; signal predicates</a></li></ol></li><li class="chapter-item expanded "><a href="timing/intro.html"><strong aria-hidden="true">5.</strong> Timing controls</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="timing/wait.html"><strong aria-hidden="true">5.1.</strong> Wait control</a></li><li class="chapter-item expanded "><a href="timing/timeout.html"><strong aria-hidden="true">5.2.</strong> Timeout control</a></li></ol></li><li class="chapter-item expanded "><li class="part-title">Appendixes</li><li class="chapter-item expanded "><a href="appendix/exit_codes.html"><strong aria-hidden="true">6.</strong> Exit codes</a></li><li class="chapter-item expanded "><a href="appendix/scripting.html"><strong aria-hidden="true">7.</strong> Advice for scripting</a></li><li class="chapter-item expanded "><a href="appendix/biblio.html"><strong aria-hidden="true">8.</strong> Bibliography</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
