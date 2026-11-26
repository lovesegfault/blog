// Formulas of the feed, with and without the signs that are not to be seen: do they look the same?
const request = new XMLHttpRequest();
request.open("GET", "/index.xml", false);
request.send();
const html = [...request.responseXML.getElementsByTagName("content:encoded")].map((it) => it.textContent).join("");
const shapes = (strip) => {
  document.documentElement.innerHTML = `<body style="font: 20px serif; background: white; color: black">${html.match(/<math.*?<\/math>/gs).map((it) => `<p>${it}</p>`).join("")}`;
  if (strip) {
    const rows = new Set([...document.querySelectorAll("mpadded[width='0']")].map((it) => it.parentNode));
    document.querySelectorAll("mpadded[width='0']").forEach((it) => it.remove());
    rows.forEach((row) => row.replaceWith(...row.children));
  }
  return [...document.querySelectorAll("p > math")].map((math) => {
    const at = math.getBoundingClientRect();
    const tokens = [...math.querySelectorAll("mi, mn, mo, mtext")].filter((it) => !it.closest("mpadded[width='0']"));
    return tokens.flatMap((it) => {
      const box = it.getBoundingClientRect();
      return [box.x + box.width / 2 - at.x, box.y - at.y, box.height];
    });
  });
};
const [bare, before, signed] = [shapes(true), [...document.body.children], shapes(false)];
const moved = signed.map((it, i) => Math.max(0, ...it.map((value, j) => Math.abs(value - bare[i][j]))));
const texts = [...document.querySelectorAll("p > math")].map((it) => it.textContent);
document.body.style.columns = 3;
[...document.body.children].forEach((it, i) => (moved[i] > 0.5 || texts[i].startsWith("u  = proj") ? it.before(before[i]) : it.remove()));
return [signed.length, moved.map((by, i) => [by.toFixed(1), texts[i].slice(0, 30)]).filter(([by]) => by > 0.5).slice(0, 40)];
