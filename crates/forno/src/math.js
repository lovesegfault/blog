// `read` and `measure` take paths in the vendor folder.

// On top of the ones that TeX loads when it needs them.
const PACKAGES = ["begingroup", "mathtools"];

globalThis.console = { log() {}, warn() {}, error() {} };
globalThis.MathJax = {
  loader: {
    paths: { mathjax: "mathjax", "mathjax-newcm": "mathjax/font" },
    require: (path) => (0, eval)(read(path)),
    load: ["input/mml", "input/tex", "output/chtml", "adaptors/liteDOM", ...PACKAGES.map((name) => `[tex]/${name}`)],
  },
  startup: { typeset: false },
  output: { displayOverflow: "linebreak", linebreaks: { lineleading: 0.5 }, mtextFont: "Spectral" },
  tex: { packages: { "[-]": ["noundefined", "require"], "[+]": PACKAGES }, formatError: (_, error) => { throw error; } },
};
MathJax.loader.require("mathjax/startup.js");

let width;
const ready = MathJax.startup.promise.then(() => {
  const { adaptor, output } = MathJax.startup;
  MathJax._.core.MmlTree.MmlNode.AbstractMmlNode.prototype.mError = (message) => { throw Error(message); };
  // Typst puts no space around brackets that grow.
  MathJax._.input.mathml.MathMLCompile.MathMLCompile.prototype.markMrows = () => {};
  // A matrix is better scrolled than broken, and a bracket stays with its exponent.
  output.factory.getNodeClass("mtable").prototype.adjustWideTable = () => {};
  output.linebreaks.FACTORS.close = (penalty, { node }) => {
    while (node.parent?.childNodes.at(-1) === node) node = node.parent;
    return node.parent?.isKind("msubsup") ? Infinity : penalty + 500;
  };
  // The space that a line ends at is not drawn, so it is not counted either.
  const space = output.factory.getNodeClass("mspace").prototype, { computeLineBBox } = space;
  space.computeLineBBox = function (line) { return Object.assign(computeLineBBox.call(this, line), { w: 0 }); };
  output.processMath = (math, parent) => {
    math.toCHTML([parent]);
    width = math.getOuterBBox().w;
  };
  output.font.cssFontMap.monospace[0] = "Berkeley Mono";
  output.measureTextNode = (node) => {
    const [family, bold, italic] = ["family", "weight", "style"].map((style) => adaptor.getStyle(node, `font-${style}`));
    const face = family.endsWith("Mono") ? "BerkeleyMono" : `Spectral-${(bold && "SemiBold") + (italic && "Italic") || "Regular"}`;
    return { w: measure(`fonts/${face}.ttf`, adaptor.textContent(node)), h: 0.75, d: 0.2 };
  };
  output.factory.getKinds().forEach((kind) => output.wrapperUsage.add(kind));
  return adaptor.cssText(MathJax.chtmlStylesheet());
});

// The widths, in ems, that a displayed formula is broken to fit: the column, a tablet's, phones', the margin.
const ROOMS = [31, 24, 18, 15.5, 13];

// With no room for aligned lines, each side gets a line to itself. Returns how far in it goes, and what it has.
function unfold(mathml) {
  const { adaptor } = MathJax.startup;
  const [math] = adaptor.tags(adaptor.body(adaptor.parse(mathml)), "math");
  const [lines, more] = math.children;
  const inside = (node) => adaptor.innerHTML(node).replace(/^(<[^>]*><\/m(i|space)>)+|(<[^>]*><\/mspace>)(?=\3)/g, "");
  if (more || lines?.kind !== "mtable") return [[0, inside(math)]];
  const sides = ([left, ...right]) => [[0, left], [1, right.filter(String).join(`<mspace width="1em"></mspace>`)]];
  return lines.children.flatMap((line) => sides(line.children.map(inside))).filter(([, side]) => side);
}

async function render(written) {
  const { adaptor, output } = MathJax.startup;
  const display = written.startsWith(`<math display`);
  let mathml = written;
  for (const [mark, tex, style] of written.matchAll(/<mi tex="(.*?)"(.*?)><\/mi>/gs)) {
    MathJax.texReset();
    MathJax.tex2mml("\\begingroupSandbox");
    const set = await MathJax.tex2mmlPromise(MathJax._.util.Entities.translate(tex), { display });
    mathml = mathml.replace(mark, () => `<mrow${style}>${set.replace(/<\/?math[^>]*>|\n */g, "")}</mrow>`);
  }
  const chancery = (_, text) => `data-mjx-variant="-tex-${text >= "𝓐" ? "bold-" : ""}calligraphic">${text.normalize("NFKC")}`;
  const tuned = mathml.replace(/mathvariant="normal">((\p{Lu}\uFE00)+)/gu, chancery).replace(/[\uFE00-\uFE0F]/g, "");
  const set = async (lines, room, how) => {
    let [html, needs] = ["", 0];
    for (const [indent, line] of lines) {
      const metrics = { display, em: 1, ex: output.font.params.x_height, containerWidth: room - indent };
      const [math] = (await MathJax.mathml2chtmlPromise(`<math ${how}>${line}</math>`, metrics)).children;
      if (indent) adaptor.setStyle(math, "margin-left", `${indent}em`);
      [html, needs] = [html + adaptor.outerHTML(math), Math.max(needs, Math.ceil(2 * (indent + width)) / 2)];
    }
    return { html, needs };
  };

  output.font.clearCache();
  output.options.linebreaks.inline = false;
  const whole = (mathml) => [[0, mathml.replace(/<\/?math[^>]*>/g, "")]];
  const commas = tuned.replaceAll(`"PUNCT"`, `"PUNCT" linebreak="goodbreak"`);
  const views = [await set(whole(tuned), 1000, display ? `display="block"` : "")];
  if (display) {
    const words = (_, open, text) => `<mrow>${open}${text.replaceAll(" ", `</mtext><mspace width=".25em"></mspace>${open}`)}</mrow>`;
    const lines = unfold(commas.replace(/(<mtext.*?>)(.*?>)/g, words));
    for (const room of ROOMS.filter((room) => room < views[0].needs)) {
      const view = await set(lines, room, `display="block" indentalign="left" indentshift="1.2em" indentshiftfirst="0"`);
      if (view.needs < views.at(-1).needs - 1) views.push(view);
    }
  } else if (width > 6) {
    output.options.linebreaks.inline = true;
    views[0] = await set(whole(width > ROOMS.at(-1) ? commas : tuned), 1000, `linebreakstyle="after"`);
  }

  // Each view is for where it fits and the one before does not.
  const styles = output.font.updateStyles({});
  const unless = (name, sign, { needs }) => {
    styles[`@container (width ${sign} ${needs}em)`] = { [`.formula .${name}-${2 * needs}`]: { display: "none" } };
    return `${name}-${2 * needs} `;
  };
  const html = views.map((view, i) => {
    const shown = (i ? unless("under", ">=", views[i - 1]) : "") + (views[i + 1] ? unless("from", "<", view) : "");
    return `<mjx-container jax="CHTML" class="${shown}" aria-hidden="true">${view.html}</mjx-container>`;
  }).join("");
  for (const [selector, style] of Object.entries(output.font.fontUsage)) {
    if (html.includes(` ${selector.match(/NCM-[\w-]+/)}"`)) styles[selector] = style;
  }
  const clean = (text) => text.replace(/ (data-[\w-]+|id)="[^"]*"| class=""/g, "");
  return [clean(html), clean(mathml), ...new MathJax._.util.StyleJson.StyleJsonSheet(styles).getStyleRules()];
}

globalThis.sheet = async () => [await ready];
globalThis.typeset = (written) => ready.then(() => render(written)).catch((error) => [error.message]);
