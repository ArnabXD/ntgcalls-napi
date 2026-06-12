// Theme toggle (dark is default), mobile-nav close, and on-page TOC scrollspy.
(() => {
  const btn = document.getElementById("theme-toggle");
  if (btn) {
    btn.addEventListener("click", () => {
      const root = document.documentElement;
      const next =
        root.getAttribute("data-theme") === "dark" ? "light" : "dark";
      root.setAttribute("data-theme", next);
      try {
        localStorage.setItem("theme", next);
      } catch (_e) {}
    });
  }

  // Close the mobile nav after following a link.
  const toggle = document.getElementById("nav-toggle");
  if (toggle) {
    for (const a of document.querySelectorAll(".sidebar .nav-link")) {
      a.addEventListener("click", () => {
        toggle.checked = false;
      });
    }
  }

  // On-page TOC scrollspy.
  const tocLinks = [...document.querySelectorAll(".toc a")];
  if (tocLinks.length && "IntersectionObserver" in window) {
    const byId = {};
    const headings = tocLinks
      .map((a) => {
        const id = decodeURIComponent(a.hash.slice(1));
        const el = document.getElementById(id);
        if (el) byId[id] = a;
        return el;
      })
      .filter(Boolean);

    const setActive = (id) => {
      for (const a of tocLinks) a.classList.remove("active");
      if (byId[id]) byId[id].classList.add("active");
    };

    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) setActive(entry.target.id);
        }
      },
      { rootMargin: "-15% 0px -75% 0px", threshold: 0 },
    );
    for (const h of headings) observer.observe(h);
  }
})();
