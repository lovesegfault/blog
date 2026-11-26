// Every page reads without this. It makes things move, and puts the notes beside the text.

const $ = (selector, within = document) => within.querySelector(selector);
const $$ = (selector, within = document) => [...within.querySelectorAll(selector)];
const on = (target, types, handle) => types.split(" ").forEach((type) => target.addEventListener(type, handle));
const [root, dark] = [document.documentElement, matchMedia("(prefers-color-scheme: dark)")];
const night = () => getComputedStyle(document.body).getPropertyValue("--scheme").includes("dark");
const calm = matchMedia("(prefers-reduced-motion: reduce)").matches;

async function load(url) {
  const { exports } = (await WebAssembly.instantiateStreaming(fetch(url))).instance;
  const send = (text) => {
    const bytes = new TextEncoder().encode(text);
    const at = exports.reserve(bytes.length);
    new Uint8Array(exports.memory.buffer).set(bytes, at);
  };
  return { ...exports, send };
}

// Formulas are drawings, and what is copied is their source.
function formulas() {
  const selection = getSelection();
  const crossed = (it) => $$("svg", it).some((part) => selection.containsNode(part));
  const picked = (it) => !selection.isCollapsed && (it.contains(selection.anchorNode) || crossed(it));
  on(document, "selectionchange", () => $$(".formula").forEach((it) => it.classList.toggle("picked", picked(it))));
  // Left to itself, a triple click stops at the first one.
  on(document, "click", ({ detail, target }) => {
    const block = target.closest("p, li, dt, dd, th, td, h1, h2, h3, h4, figcaption, figure");
    if (detail === 3 && block?.matches(":not(figure)") && $(".formula", block)) selection.selectAllChildren(block);
  });
  on(document, "copy", () => {
    const [touched, at] = [$$(".formula").filter((it) => selection.containsNode(it, true)), scrollY];
    if (!touched.length) return;
    // Firefox remembers where a range was, and so do its clones.
    const ranges = Array.from({ length: selection.rangeCount }, (_, i) => {
      const [range, was] = [new Range(), selection.getRangeAt(i)];
      range.setStart(was.startContainer, was.startOffset);
      range.setEnd(was.endContainer, was.endOffset);
      return range;
    });
    const select = () => (selection.removeAllRanges(), ranges.forEach((range) => selection.addRange(range)));
    for (const it of touched) {
      const [start, end] = it.matches(".picked") ? ["Before", "After"] : ["After", "Before"];
      for (const range of ranges) {
        if (it.contains(range.startContainer)) range[`setStart${start}`](it);
        if (it.contains(range.endContainer)) range[`setEnd${end}`](it);
      }
      it.prepend(it.dataset.source);
      it.classList.add("copied");
    }
    select();
    requestAnimationFrame(() => {
      for (const it of touched) it.firstChild.remove(), it.classList.remove("copied");
      select();
      scrollTo(scrollX, at);
    });
  });
}

function dusk(button) {
  const show = (theme = "") => {
    root.dataset.theme = theme;
    dark.dispatchEvent(new Event("change"));
  };
  on(button, "click", () => {
    show(night() !== dark.matches ? "" : night() ? "light" : "dark");
    try { localStorage.theme = root.dataset.theme; } catch {}
  });
  // It may have changed on another page.
  on(window, "pageshow storage", () => {
    try { show(localStorage.theme); } catch {}
  });
}

function comments(section) {
  const script = document.importNode($("template", section).content.firstChild);
  const { origin } = new URL(script.src);
  const theme = () => (section.classList.toggle("night", night()) ? "gruvbox-dark" : "github-light");
  const match = () => $("iframe", section).contentWindow.postMessage({ type: "set-theme", theme: theme() }, origin);
  script.setAttribute("theme", theme());
  section.append(script);
  on(window, "message", (event) => {
    if (event.origin !== origin || section.matches(".live")) return;
    section.classList.add("live");
    on(dark, "change", match);
    match();
  });
}

async function figure(element) {
  const [canvas, mural, tiles] = [$("canvas", element), $(".mural", element), new Image()];
  tiles.src = getComputedStyle(mural).backgroundImage.slice(5, -2);
  const [program] = await Promise.all([load(element.dataset.program), tiles.decode()]);
  program.send(element.dataset.event);
  program.collide();

  const pen = canvas.getContext("2d");
  const posts = element.dataset.event.split("|").slice(1).map((post) => post.split(" ")[0]);
  const light = (names) => posts.map((post) => (!names || names.split(" ").includes(post) ? 1 : 0.1));
  const born = calm ? -Infinity : performance.now();
  let [wanted, shown] = [light(element.dataset.lit), light(element.dataset.lit)];
  let [turned, drawn, waiting, visible, held, stirred] = [0, 0, false, false, null, born];
  let place, across, fire, weight, density;

  function measure() {
    const [box, wall, style] = [element.getBoundingClientRect(), mural.getBoundingClientRect(), getComputedStyle(element)];
    const value = (name) => parseFloat(style.getPropertyValue(name));
    const unit = wall.width / value("--w");
    const [x, y] = [wall.x - box.x + value("--vx") * unit, wall.y - box.y + value("--vy") * unit];
    const [radius, cos, sin] = [value("--radius") * unit, Math.cos(value("--turn")), Math.sin(value("--turn"))];
    const colours = style.getPropertyValue("--fire").trim().split(" ");
    place = (a, b) => [x + radius * (a * cos - b * sin), y + radius * (a * sin + b * cos)];
    across = (dx, dy) => (dy * cos - dx * sin) / radius;
    density = Math.min(2, devicePixelRatio);
    weight = value("--weight") * Math.sqrt(unit) * (density < 1.5 ? 1.15 : 1);
    [canvas.width, canvas.height] = [box.width * density, box.height * density];
    fire = [0.3, 0.3, 0.14].map((middle, kind) => {
      const gradient = pen.createRadialGradient(x, y, 0, x, y, radius * 1.9);
      [0, middle, 1].forEach((stop, i) => gradient.addColorStop(stop, colours[kind * 3 + i]));
      return gradient;
    });
  }

  function draw(age) {
    const start = program.frame(turned, 1 - (1 - Math.min(1, age)) ** 3.2);
    const frame = new Float32Array(program.memory.buffer, start);
    const [tracks, shadows, hits] = [[], posts.map(() => new Path2D()), posts.map(() => [0, 1, 2].map(() => new Path2D()))];
    let i = 2;
    for (let left = frame[0]; left--; ) {
      const [post, kind, width, opacity, points] = frame.subarray(i, (i += 5));
      const path = new Path2D();
      for (const end = i + points * 2; i < end; i += 2) path.lineTo(...place(frame[i], frame[i + 1]));
      shadows[post].addPath(path);
      tracks.push([path, fire[kind], width, opacity * shown[post]]);
    }
    for (let left = frame[1]; left--; i += 4) {
      const [x, y] = place(frame[i + 2], frame[i + 3]);
      hits[frame[i]][frame[i + 1]].moveTo(x, y);
      hits[frame[i]][frame[i + 1]].lineTo(x + 0.01, y);
    }
    const stroke = ([path, colour, width, opacity]) => {
      [pen.strokeStyle, pen.lineWidth, pen.globalAlpha] = [colour, width * weight, opacity];
      pen.stroke(path);
    };
    const landed = Math.max(0, Math.min(1, age * 2 - 0.6));
    pen.clearRect(0, 0, canvas.width / density, canvas.height / density);
    pen.lineCap = pen.lineJoin = "round";
    pen.setTransform(density, 0, 0, density, 0.7 * density, 1.1 * density);
    shadows.forEach((path, post) => stroke([path, "#020624", 1.7, 0.45 * shown[post]]));
    pen.setTransform(density, 0, 0, density, 0, 0);
    hits.forEach((sizes, post) => sizes.forEach((path, size) => {
      stroke([path, "#e4ecff", [1.6, 2.3, 3.3][size], [0.55, 0.75, 0.92][size] * landed * shown[post]]);
    }));
    tracks.forEach(stroke);
  }

  function tick(now) {
    const age = Math.max(0, now - born) / 1700;
    const busy = age < 1 || held || shown.some((level, post) => Math.abs(wanted[post] - level) > 0.004);
    // It comes to rest when no one is there.
    const drift = calm || held ? 0 : Math.max(0, Math.min(1, 10 - (now - stirred) / 3000));
    waiting = false;
    // Drifting is slow enough for every other frame.
    if (busy || now - drawn > 30) {
      turned += Math.min(80, now - drawn) * 0.000035 * drift;
      shown = shown.map((level, post) => level + (wanted[post] - level) * (calm ? 1 : 0.2));
      draw(age);
      drawn = now;
    }
    if (visible && (busy || drift)) wake();
  }
  const wake = () => (waiting ||= requestAnimationFrame(tick));

  new ResizeObserver(() => {
    cancelAnimationFrame(waiting);
    measure();
    drawn = 0;
    tick(performance.now());
  }).observe(element);
  new IntersectionObserver(([entry]) => (visible = entry.isIntersecting) && wake()).observe(element);
  on(window, "pointermove keydown scroll focus", () => ((stirred = performance.now()), visible && !calm && wake()));
  on(visualViewport, "resize", () => (element.style.touchAction = visualViewport.scale > 1.01 ? "auto" : ""));
  on(element, "pointerdown", (event) => {
    if (event.button || event.target.closest("a")) return;
    element.setPointerCapture(event.pointerId);
    held = event;
    wake();
  });
  on(element, "pointermove", (event) => {
    if (event.pointerId !== held?.pointerId) return;
    turned += 0.9 * across(event.clientX - held.clientX, event.clientY - held.clientY);
    held = event;
  });
  on(element, "pointerup pointercancel contextmenu", () => (held = null));
  for (const link of $$("[data-lights]")) {
    on(link, "mouseenter focus", () => wake((wanted = light(link.dataset.lights))));
    on(link, "mouseleave blur", () => wake((wanted = light(element.dataset.lit))));
  }
  element.classList.add("live");
}

function contents(nav) {
  const links = $$("a", nav);
  const headings = links.map((link) => document.getElementById(decodeURIComponent(link.hash.slice(1))));
  const follow = () => {
    const current = headings.findLastIndex((heading) => heading.getBoundingClientRect().top < 140);
    links.forEach((link, i) => (link.ariaCurrent = i === current ? "true" : null));
  };
  on(window, "scroll", follow);
  follow();
}

function notes(section) {
  const prose = section.parentElement;
  const inMargin = () => getComputedStyle(section).position === "absolute";
  // Each note goes level with its mark, clear of the one before and of anything else in the margin.
  const place = () => {
    if (!inMargin()) return prose.style.removeProperty("min-height");
    const origin = prose.getBoundingClientRect().top;
    const taken = $$(":is(.wide, .margin):not(.notes *)", prose).map((other) => other.getBoundingClientRect());
    let floor = origin;
    for (const note of $$(":scope > ol > li", section)) {
      const mark = document.getElementById(note.id.replace("fn", "fnref"));
      const [height, raised] = [note.offsetHeight + 14, mark.closest(".code") ? -1 : 4];
      let y = Math.max(floor, (mark.closest("details:not([open])") ?? mark).getBoundingClientRect().top + raised);
      for (const { top, bottom } of taken) {
        if (y < bottom && y + height > top) y = y < top && top - height >= floor ? top - height : bottom + 14;
        if (bottom <= y) floor = Math.max(floor, bottom + 14);
      }
      note.style.setProperty("--y", `${y - origin}px`);
      floor = y + height;
    }
    prose.style.minHeight = `${floor - origin}px`;
  };
  section.classList.add("live");
  place();
  $(":target", section)?.scrollIntoView();
  const sizes = new ResizeObserver(place);
  [prose, ...$$(".margin, .notes > ol > li", prose)].forEach((it) => sizes.observe(it));
  document.fonts.ready.then(place);
  // With no margin, a note comes up over the page instead of sending the reader to the end of it.
  const peek = Object.assign(document.createElement("div"), { className: "peek", popover: "auto", tabIndex: -1 });
  let lent, source;
  const restore = () => (lent.append(...peek.childNodes), lent.style.removeProperty("height"));
  prose.append(peek);
  on(window, "scroll resize", () => peek.togglePopover?.(false));
  on(peek, "toggle", () => peek.matches(":popover-open") || restore());
  for (const link of $$(".ref a", prose)) {
    const note = document.getElementById(link.hash.slice(1));
    on(link, "mouseenter focus", () => note.classList.add("on"));
    on(link, "mouseleave blur", () => note.classList.remove("on"));
    on(link, "click", (event) => {
      if (inMargin()) {
        note.scrollIntoView({ block: "nearest", behavior: calm ? "auto" : "smooth" });
        Object.assign(note, { tabIndex: -1 }).focus({ preventScroll: true });
      } else {
        if (!peek.contains(link)) source = link;
        peek.togglePopover(false);
        if (lent) restore();
        note.style.height = getComputedStyle(note).height;
        peek.append(...(lent = note).childNodes);
        peek.showPopover({ source });
        peek.focus({ preventScroll: true });
      }
      event.preventDefault();
    });
  }
}

function code(block) {
  const [lines, all, patch] = [$(".lines", block), $$(".l", block), block.matches(".patch")];
  block.insertAdjacentHTML("beforeend", "<div class=tools><button>wrap</button><button aria-live=polite>copy</button></div>");
  const [wrap, copy] = $$(".tools button", block);
  const wanted = patch || !$(".cmd, .del", block) ? all : $$(".l:not(.del, .out)", block);
  const source = wanted.map((line) => line.children[1].textContent).join("").replace(patch ? "" : /\s+$/, "");
  const { language, options } = block.dataset;
  if (language) {
    const executors = [{ compiler: { options }, compilerOutputVisible: true }];
    const state = JSON.stringify({ sessions: [{ id: 1, language, source, compilers: [{ options }], executors }] });
    const ascii = state.replace(/[^\0-~]/g, (c) => `\\u${c.charCodeAt().toString(16).padStart(4, 0)}`);
    const href = `https://godbolt.org/clientstate/${btoa(ascii).replaceAll("/", "_")}`;
    copy.before(Object.assign(document.createElement("a"), { href, text: "run", title: "On Compiler Explorer" }));
  }
  const cut = () => lines.scrollWidth > lines.clientWidth;
  new ResizeObserver(() => {
    const row = parseFloat(getComputedStyle(lines).lineHeight);
    [wrap.hidden, wrap.ariaPressed] = [!cut() && all.every((line) => line.offsetHeight < 1.5 * row), !cut()];
  }).observe(lines);
  on(lines, "scroll", () => lines.classList.toggle("slid", lines.scrollLeft > 0));
  on(lines, "click", (event) => {
    if (!event.target.matches("a.at")) return;
    const at = scrollY;
    event.preventDefault();
    location.replace(event.target.hash === location.hash ? `#${block.id}` : event.target.hash);
    scrollTo(scrollX, at);
  });
  on(wrap, "click", () => {
    wrap.ariaPressed = block.classList.toggle("wrap", cut());
    lines.style.setProperty("--columns", 0);
    for (const line of all) line.style.setProperty("--hang", line.textContent.search(/\S|$/) + 2);
  });
  on(copy, "click", async () => {
    await navigator.clipboard.writeText(source);
    copy.textContent = "copied";
    setTimeout(() => (copy.textContent = "copy"), 1200);
  });
}

// Changes `node` into `wanted` in place, so that focus and dragging carry on.
function morph(node, wanted) {
  const children = [...wanted.childNodes];
  while (node.childNodes.length > children.length) node.lastChild.remove();
  children.forEach((child, i) => {
    const old = node.childNodes[i];
    if (!old) return node.append(child);
    if (old.nodeName !== child.nodeName) return old.replaceWith(child);
    if (!old.attributes) return old.data === child.data || (old.data = child.data);
    for (const { name } of [...old.attributes]) if (!child.hasAttribute(name)) old.removeAttribute(name);
    for (const { name, value } of child.attributes) if (old.getAttribute(name) !== value) old.setAttribute(name, value);
    const { value, checked } = child;
    morph(old, child);
    if (old.matches("input:not(:focus:read-write), select:not(:focus)")) Object.assign(old, { value, checked });
  });
}

async function explainer(element) {
  const [program, template] = [await load(element.dataset.program), document.createElement("template")];
  let [last, waiting, moving, visible, held] = [null, false, !calm, true, null];
  // By name, as another element may have taken its place.
  const handle = (name) => $(`[data-drag="${name}"]`, element);
  const step = (now) => {
    moving = program.tick(Math.min(100, now - (last ?? now)) / 1000);
    const [length, focused] = [program.view(), $("[data-drag]:focus", element)?.dataset.drag];
    template.innerHTML = new TextDecoder().decode(new Uint8Array(program.memory.buffer, program.text(), length));
    morph(element, template.content);
    handle(focused)?.focus({ preventScroll: true });
    [last, waiting] = moving && visible ? [now, requestAnimationFrame(step)] : [null, false];
  };
  const wake = () => (waiting ||= requestAnimationFrame(step));
  const input = (name, value) => {
    program.send(`${name}=${value}`);
    program.input();
    wake();
  };
  const drag = (handle, x, y) => {
    const point = new DOMPoint(x, y).matrixTransform(handle.ownerSVGElement.getScreenCTM().inverse());
    input(handle.dataset.drag, `${point.x} ${point.y}`);
  };
  program.send(element.dataset.options);
  program.start();
  new IntersectionObserver(([entry]) => (visible = entry.isIntersecting) && moving && wake()).observe(element);
  on(element, "submit", (event) => event.preventDefault());
  on(element, "focusout", wake);
  on(element, "touchstart", (event) => event.target.closest("[data-drag]") && event.preventDefault());
  on(element, "input", ({ target }) => input(target.name, target.type === "checkbox" ? target.checked : target.value));
  on(element, "click", ({ target }) => {
    const button = target.closest("button");
    if (button) input(button.name, button.value);
  });
  on(element, "pointerdown", (event) => {
    held = event.button ? null : event.target.closest("[data-drag]")?.dataset.drag;
    if (held) element.setPointerCapture(event.pointerId);
  });
  on(element, "pointermove", (event) => handle(held) && drag(handle(held), event.clientX, event.clientY));
  on(element, "pointerup pointercancel contextmenu", () => (held = null));
  on(element, "keydown", (event) => {
    const nudge = { ArrowLeft: [-8, 0], ArrowRight: [8, 0], ArrowUp: [0, -8], ArrowDown: [0, 8] }[event.key];
    if (!nudge || !event.target.dataset.drag) return;
    const { x, y, width, height } = event.target.getBoundingClientRect();
    event.preventDefault();
    drag(event.target, x + width / 2 + nudge[0], y + height / 2 + nudge[1]);
  });
  element.classList.add("live");
}

$$(".comments").forEach(comments);
$$(".dusk").forEach(dusk);
$$(".contents").forEach(contents);
$$(".notes").forEach(notes);
$$(".code").forEach(code);
$$(".explainer").forEach(explainer);
$$(".figure").forEach((element) => figure(element).catch(() => {
  const still = $("noscript", element);
  still.outerHTML = still.textContent;
}));

// Whatever scrolls sideways has to be reachable with the keyboard.
const scrollers = new ResizeObserver((entries) => {
  for (const { target } of entries) {
    if (target.scrollWidth > target.clientWidth) target.tabIndex = 0;
    else target.removeAttribute("tabindex");
  }
});
$$(".lines, .display > .formula, .table").forEach((scroller) => scrollers.observe(scroller));
formulas();
