// Run in a page that frames a post. At every width: is each formula in order, inside its column, and only broken if it has to be?
const frame = document.querySelector('iframe'), d = frame.contentDocument, found = {}, r = e => e.getBoundingClientRect();
const box = (n, at) => { const g = d.createRange(); g.setStart(n, at); g.setEnd(n, at + 1); return r(g); };
const note = (what, f, width) => (found[what + ' ' + f.dataset.source.slice(0, 26)] ??= []).push(width);
for (let width = 320; width < 1500; width += width < 700 ? 7 : 61) {
  frame.style.width = width + 'px';
  for (const f of d.querySelectorAll('.prose .formula')) {
    const parts = [...f.querySelectorAll('b > svg')].map(r), set = f.firstChild, em = parseFloat(d.defaultView.getComputedStyle(f).fontSize);
    if (!parts.length || !f.checkVisibility()) continue;
    if (parts.some((p, i) => i && (p.bottom < parts[i - 1].bottom - 1 || (p.bottom < parts[i - 1].bottom + 1 && p.left < parts[i - 1].right - 1)))) note('disorder', f, width);
    if (f.parentElement.matches('.display')) {
      const lines = new Set(parts.map(p => Math.round(p.bottom))).size, fits = parseFloat(set.style.getPropertyValue('--wide')) * em + 3 < f.clientWidth;
      if (fits && lines > set.children.length) note('needless', f, width);
      if (f.scrollWidth > f.clientWidth) note('scrolls', f, width);
      if (parts.some(p => p.left < r(f).left - 1)) note('cut off', f, width);
      continue;
    }
    let [a, b] = [f.previousSibling, f.nextSibling];
    while (a?.matches?.('a, em, strong, sup, code')) a = a.lastChild;
    while (b?.matches?.('a, em, strong, sup, code')) b = b.firstChild;
    if (/[^\s—–-]$/.test(a?.data ?? ' ') && box(a, a.data.length - 1).left > parts[0].left) note('before', f, width);
    if (/^[^\s—]/.test(b?.data ?? ' ') && box(b, 0).left < parts.at(-1).left) note('after', f, width);
    if (parts.some(p => p.right > r(f.closest('p, li, td, figcaption, h2, h3, h4, div')).right + 2)) note('spills', f, width);
  }
}
return Object.entries(found).map(([what, at]) => `${what} @${at[0]}-${at.at(-1)} x${at.length}`);
