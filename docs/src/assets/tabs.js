// Tabbed code blocks. Tabs sharing a data-tab-group sync across the page and
// persist the choice in localStorage.
(() => {
  const groups = [...document.querySelectorAll(".tabs")];
  if (!groups.length) return;

  const keyFor = (group) => `tab:${group}`;

  const select = (group, slug, persist) => {
    for (const tabs of document.querySelectorAll(
      `.tabs[data-tab-group="${group}"]`,
    )) {
      const has = [...tabs.querySelectorAll(".tab-btn")].some(
        (b) => b.dataset.tab === slug,
      );
      if (!has) continue; // this group instance doesn't offer that tab
      for (const btn of tabs.querySelectorAll(".tab-btn")) {
        btn.setAttribute("aria-selected", btn.dataset.tab === slug);
      }
      for (const panel of tabs.querySelectorAll(".tab-panel")) {
        panel.hidden = panel.dataset.tab !== slug;
      }
    }
    if (persist) {
      try {
        localStorage.setItem(keyFor(group), slug);
      } catch (_e) {}
    }
  };

  for (const tabs of groups) {
    const group = tabs.dataset.tabGroup;
    const btns = [...tabs.querySelectorAll(".tab-btn")];
    let initial = btns[0]?.dataset.tab;
    try {
      const saved = localStorage.getItem(keyFor(group));
      if (saved && btns.some((b) => b.dataset.tab === saved)) initial = saved;
    } catch (_e) {}
    if (initial) select(group, initial, false);

    for (const btn of btns) {
      btn.addEventListener("click", () => select(group, btn.dataset.tab, true));
    }
  }
})();
