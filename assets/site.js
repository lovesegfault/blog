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
  const [origin, script, { repo, issue }] = ["https://utteranc.es", document.createElement("script"), section.dataset];
  const theme = () => (section.classList.toggle("night", night()) ? "gruvbox-dark" : "github-light");
  const match = () => $("iframe", section).contentWindow.postMessage({ type: "set-theme", theme: theme() }, origin);
  const thread = issue ? ["issue-number", issue] : ["issue-term", "title"];
  for (const pair of [["repo", repo], thread, ["theme", theme()]]) script.setAttribute(...pair);
  section.append(Object.assign(script, { src: `${origin}/client.js`, crossOrigin: "anonymous" }));
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

// The bed of the lake goes over the water and the tracks, where it is dry. Lengths are the mural's own pixels.
const CORNERS = `#version 300 es
void main() { gl_Position = vec4(gl_VertexID % 2 * 4 - 1, gl_VertexID / 2 * 4 - 1, 0, 1); }`;
const SHORE = `#version 300 es
precision highp float;
precision highp sampler2D;
uniform sampler2D bed, chart, lustre;
uniform vec4 wall, arrow, ink, drop;
uniform vec3 setting;
out vec4 colour;

vec4 look(sampler2D it, vec2 at) { return textureLod(it, at / wall.zw, log2(.8 * vec2(textureSize(it, 0)).x / wall.x)); }
float depth(vec2 at) { return (dot(look(chart, at).rb, vec2(255, 1)) - 25.) / 170.; }
float ramp(float from, float to, float it) { return smoothstep(0., 1., (it - from) / (to - from)); }
float fade(float it, float over) { return exp(-abs(it) / over); }
float wander(vec2 at) { return sin(at.x + 1.7 * sin(at.y * .8)) * sin(at.y + 1.7 * sin(at.x * .6)); }
void lay(vec3 paint, float share) { colour = mix(colour, vec4(paint * setting.z, 1), share); }

void main() {
  vec2 at = vec2(gl_FragCoord.x, wall.y - gl_FragCoord.y) / wall.xy * wall.zw, from = at - arrow.xy, apart = vec2(4, 0);
  float glaze = look(lustre, at).a, dots = wall.x / wall.z, brush = sqrt(setting.y / 62.), was = drop.z, time = drop.w;
  colour = look(bed, at);
  if (colour.a + glaze == 0.) return;

  // It moves from the collision outwards, and fastest over flat ground.
  float level = clamp(mix(drop.x, drop.y, ramp(0., 11., length(from) / setting.y)), 0., 1.), deep = depth(at);
  vec2 rise = vec2(depth(at + apart) - depth(at - apart), depth(at + apart.yx) - depth(at - apart.yx));
  float slope = max(length(rise) / 8., 5e-4) * brush;
  float stir = 1. - fade(level - was, .045), rush = 1. - fade(level - was, 40. * slope);
  // At either end, the shore is the one that was fired.
  float low = ramp(0., .015, level), high = ramp(1., .97, level), over = deep - level - 3. * slope * (1. - high);
  vec2 lap = vec2(wander(at / 23. + time * .9), wander(at.yx / 29. - time * 1.1));
  float off = over / slope + (1. + 2.5 * rush) * stir * lap.x, wet = ramp(-.7, .7, off * brush * dots);

  // What it has just left is damp, and what it has just covered shows through.
  vec4 land = look(bed, at + 1.6 * wet * stir * lap);
  vec4 drowned = land * vec4(.34, .56, .72, 1) + land.a * setting.z * vec4(.03, .13, .36, 0);
  float damp = stir * clamp((deep - was) / max(level - was, .001), 0., 1.);
  float sunk = stir * clamp((was - deep) / max(was - level, .001), 0., 1.);
  float clear = max(.4 * ramp(0., .15, level) * (fade(off, 9.) + fade(over, .08 + .3 * stir)), .9 * sqrt(sunk));
  colour = mix(land * (1. - damp * vec4(.24, .3, .36, 0)), drowned, .6 * damp * fade(off, 9.)) * low;
  colour = mix(colour, drowned * clear, wet);

  // The shaft of the arrow, and the brushed line of the shore.
  vec2 along = vec2(dot(from, arrow.zw), from.y * arrow.z - from.x * arrow.w) * setting.x;
  float dash = min(min(3.5 - abs(mod(along.x - 4., 13.) - 6.5), .675 - abs(along.y)), along.x) * dots / setting.x;
  float load = 1. + .5 * wander(at / 41.), bare = max(glaze, ramp(.12, .06, deep) * land.a) * low;
  lay(ink.rgb / setting.z, ink.a * land.a * low * (1. - wet) * clamp(dash + .5, 0., 1.));
  lay(vec3(.29, .165, .08), .24 * bare * high * fade(off, 3.4));
  lay(vec3(.14, .075, .05), .85 * bare * high * ramp(.7, -.6, abs(off) - load) * (1. - .8 * stir));

  // Light on the tiles, ripples and foam.
  vec2 tile = floor(from / setting.y + .5);
  float surge = drop.x - drop.y, far = length(tile);
  float glint = sin(far * .9 + sign(surge) * time * 7. + 1.3 * sin(dot(tile, vec2(12.9, 78.2))));
  float swell = over * 9. + time * .9, lace = ramp(-.6, .3, wander(at / 19. + 3. * floor(swell)));
  float crest = ramp(1.2, .3, abs(fract(swell) - .5) / 9. / slope) * ramp(.03, .015, slope) * lace;
  float wide = .5 + rush * (3.2 + 2.8 * lap.y) * (1. - .55 * damp);
  float foam = ramp(-1., .2, off) * max(.9 * ramp(.8, -.8, off - wide), .4 * fade(off, 2. * wide));
  lay(vec3(.36, .58, 1), .26 * glaze * wet * ramp(0., 1., glint) * ramp(0., 3., far) * (1. - fade(surge, .06)));
  lay(vec3(.7, .84, 1), .75 * glaze * wet * stir * crest * fade(over, .2));
  lay(vec3(.98, .96, .88), bare * stir * foam);
}`;

function tide(element) {
  const [canvas, arrow, rising] = [$(".bed", element), $(".arrow", element), element.dataset.water === "rising"];
  // How far the water is down: by the collision, away from it, and a moment ago.
  const drop = [0, 0, 0];
  let [own, pace, speed, time] = [rising ? +!calm : 1 - (element.dataset.water ?? 1), 2.6, 0, 0];
  let gl, set, asked, laid, wanted, drawn, waiting, visible;

  function compile() {
    const program = gl.createProgram();
    for (const [kind, source] of [[gl.VERTEX_SHADER, CORNERS], [gl.FRAGMENT_SHADER, SHORE]]) {
      const shader = gl.createShader(kind);
      gl.shaderSource(shader, source);
      gl.compileShader(shader);
      gl.attachShader(program, shader);
    }
    gl.linkProgram(program);
    gl.useProgram(program);
    gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL, true);
    ["bed", "chart", "lustre"].forEach((name, unit) => {
      gl.uniform1i(gl.getUniformLocation(program, name), unit);
      gl.activeTexture(gl.TEXTURE0 + unit);
      gl.bindTexture(gl.TEXTURE_2D, gl.createTexture());
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR_MIPMAP_LINEAR);
      for (const side of [gl.TEXTURE_WRAP_S, gl.TEXTURE_WRAP_T]) gl.texParameteri(gl.TEXTURE_2D, side, gl.CLAMP_TO_EDGE);
    });
    set = (name, ...values) => gl[`uniform${values.length}f`](gl.getUniformLocation(program, name), ...values);
    asked = null;
  }

  async function measure() {
    const value = (name) => getComputedStyle(element).getPropertyValue(`--${name}`).trim();
    const picture = (name) => Object.assign(new Image(), { src: value(name).replace(/^url\(|["\\)]/g, "") });
    if (!canvas.offsetWidth) return;
    if (asked !== value("bed")) {
      const [bed, pictures] = [(asked = value("bed")), ["bed", "depth", "water"].map(picture)];
      laid = gl.clear(gl.COLOR_BUFFER_BIT);
      await Promise.all(pictures.map((it) => it.decode())).catch(() => canvas.remove());
      if (asked !== bed || !canvas.isConnected || gl.isContextLost()) return;
      pictures.forEach((it, unit) => {
        gl.activeTexture(gl.TEXTURE0 + unit);
        gl.pixelStorei(gl.UNPACK_COLORSPACE_CONVERSION_WEBGL, unit ? gl.NONE : gl.BROWSER_DEFAULT_WEBGL);
        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, it);
        gl.generateMipmap(gl.TEXTURE_2D);
      });
      // It goes on from what was to be seen.
      drop.fill(rising && !canvas.matches(".live") ? 1 : 0);
      canvas.classList.add("live");
      [laid] = pictures;
    }
    if (!laid) return;
    const [w, h, x, y, turn, tile] = ["w", "h", "vx", "vy", "turn", "tile"].map((name) => parseFloat(value(name)));
    const unit = canvas.getBoundingClientRect().width / w;
    const dots = Math.min(unit * Math.min(2, devicePixelRatio), laid.width / w);
    [canvas.width, canvas.height] = [w * dots, h * dots].map(Math.round);
    gl.viewport(0, 0, canvas.width, canvas.height);
    set("wall", canvas.width, canvas.height, w, h);
    set("arrow", x, y, Math.cos(turn), Math.sin(turn));
    set("setting", unit, tile, night() ? 0.6 : 0.94);
    cancelAnimationFrame(waiting);
    tick(drawn);
  }

  function tick(now) {
    // Rising is slow enough for every other frame.
    if (rising && now > drawn && now < drawn + 30) return (waiting = requestAnimationFrame(tick));
    const [passed, { color, opacity }] = [Math.min(60, now - drawn || 0) / 1000, getComputedStyle(arrow)];
    [drawn, waiting] = [now, false];
    if (!laid) return;
    if (rising) own = Math.max(0, own - passed / 40);
    const goal = wanted ?? own;
    speed += (pace * (goal - drop[0]) - 1.6 * speed) * pace * passed;
    drop[0] += speed * passed;
    drop[1] += (drop[0] - drop[1]) * 4 * passed;
    drop[2] += (drop[1] - drop[2]) * 1.6 * passed;
    const still = calm || (!(rising && own) && drop.every((it) => Math.abs(it - goal) < 0.0004));
    if (still) drop.fill(goal);
    set("drop", ...drop, (time += passed));
    set("ink", ...color.match(/\d+/g).slice(0, 3).map((it) => it / 255), opacity);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
    if (visible && !(still && +opacity === 1)) wake();
  }
  const wake = () => (waiting ||= requestAnimationFrame(tick));

  function start() {
    if (gl !== undefined) return;
    const modest = { antialias: false, depth: false, powerPreference: "low-power", failIfMajorPerformanceCaveat: true };
    gl = canvas.getContext("webgl2", modest);
    if (!gl || gl.isContextLost()) return canvas.remove();
    compile();
    new ResizeObserver(measure).observe(canvas);
    new IntersectionObserver(([entry]) => (visible = entry.isIntersecting) && wake()).observe(element);
    on(dark, "change", measure);
    on(canvas, "webglcontextlost", (event) => (event.preventDefault(), (canvas.hidden = true), (laid = null)));
    on(canvas, "webglcontextrestored", () => (compile(), (canvas.hidden = false), measure()));
  }

  const posts = $$("a[data-water]");
  for (const post of posts) {
    on(post, "pointerenter focus", ({ pointerType }) => {
      if (pointerType ? pointerType === "touch" : !post.matches(":focus-visible")) return;
      [wanted, pace] = [1 - (+post.dataset.water || 0), 4.4];
      wake(start());
    });
    on(post, "pointerleave blur", () => wake((wanted = null)));
  }
  // Once the tracks are out.
  setTimeout(() => {
    if (own) start();
    if (posts.length) on(window, "pointermove", ({ pointerType }) => pointerType === "touch" || start());
  }, calm ? 0 : 1200);
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
}).then(() => tide(element)));

// Whatever scrolls sideways has to be reachable with the keyboard.
const scrollers = new ResizeObserver((entries) => {
  for (const { target } of entries) {
    if (target.scrollWidth > target.clientWidth) target.tabIndex = 0;
    else target.removeAttribute("tabindex");
  }
});
$$(".lines, .display > .formula, .table").forEach((scroller) => scrollers.observe(scroller));
formulas();
