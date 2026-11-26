"""Temporary: looks at the preview through a WebDriver. Pages are framed, so that any width fits the screen."""

import base64
import json
import os
import subprocess
import sys
import time
import urllib.request

SITE = "http://127.0.0.1:1313"
CHECKS = """
const w = document.querySelector('iframe').contentWindow, d = w.document, r = e => e.getBoundingClientRect();
const all = s => [...d.querySelectorAll(s)], c = d.querySelector('canvas');
let ink; try { ink = c.getContext('2d').getImageData(0, 0, c.width, c.height).data.filter((v, i) => i % 4 == 3 && v).length; } catch (e) { ink = `${e} ${c.width}x${c.height}`; }
const line = (n, at) => { const g = d.createRange(); g.setStart(n, at); g.setEnd(n, at + 1); return r(g); };
const code = d.querySelector('#code1 .lines');
if (code) w.getSelection().selectAllChildren(code);
const picked = code && w.getSelection().toString();
w.getSelection().removeAllRanges();
return {
  asleep: all('.figure, .notes, .explainer').filter(e => !e.matches('.live')).length,
  ink,
  fonts: [...new Set([...d.fonts].filter(f => f.status == 'loaded').map(f => f.family))].join(),
  sideways: d.documentElement.scrollWidth - d.documentElement.clientWidth,
  cut: all('main :is(p, li, h1, figure, .display)').filter(e => r(e).width && (r(e).right > w.innerWidth + 1 || r(e).left < -1)).length,
  notes: all('.notes > ol > li').map(n => Math.round(r(n).top - r(d.getElementById(n.id.replace('fn', 'fnref'))).top)).join(),
  contents: all('.contents a').filter(a => a.checkVisibility()).length,
  scrolling: all('.lines').filter(l => l.scrollWidth > l.clientWidth).length,
  gutter: code && w.getComputedStyle(code).getPropertyValue('--gutter'),
  picked: picked && [picked.trim().split('\\n').length, picked.slice(0, 40)].join(),
  blank: all('.formula use').filter(u => u.closest('svg').checkVisibility() && !r(u).width).length,
  sliding: all('.display > .formula').filter(f => f.scrollWidth > f.clientWidth).length,
  orphans: all('p > .formula, li > .formula').filter(k => /^\\S/.test(k.nextSibling?.data ?? '') && line(k.nextSibling, 0).top > [...k.getClientRects()].at(-1).bottom - 4).length,
  broken: all('p > .formula, li > .formula').filter(k => r(k).height > 45).length,
  spilling: all('.prose :is(p, li)').filter(p => p.scrollWidth > p.clientWidth + 40).length,
};
"""


def call(method, path, body=None):
    data = json.dumps(body).encode() if body is not None else None
    request = urllib.request.Request("http://127.0.0.1:4444" + path, data, {"Content-Type": "application/json"}, method=method)
    try:
        return json.load(urllib.request.urlopen(request))["value"]
    except urllib.error.HTTPError as error:
        return error.read().decode()[:400]


os.makedirs("shots", exist_ok=True)
if sys.argv[1] == "webkit":
    from playwright.sync_api import sync_playwright

    browser = sync_playwright().start().webkit.launch()
    print("webkit", browser.version)
    page = browser.new_page(viewport={"width": 1440, "height": 1000}, device_scale_factor=2)
    go, run, shot = page.goto, lambda script: page.evaluate("() => {" + script + "}"), page.screenshot
    press = lambda key: page.keyboard.press("ControlOrMeta+" + key)
else:
    driver = subprocess.Popen([*sys.argv[1].split(), "--port=4444"], stdout=subprocess.DEVNULL)
    time.sleep(3)
    started = call("POST", "/session", {"capabilities": {}})
    session = "/session/" + started["sessionId"]
    print(started["capabilities"]["browserName"], started["capabilities"]["browserVersion"])
    print(call("POST", session + "/window/rect", {"x": 0, "y": 0, "width": 1500, "height": 1100}))
    call("POST", session + "/timeouts", {"script": 200000})
    go = lambda url: call("POST", session + "/url", {"url": url})
    run = lambda script: call("POST", session + "/execute/sync", {"script": script, "args": []})
    held = "\ue03d" if sys.platform == "darwin" else "\ue009"
    keys = lambda key: [{"type": how, "value": which} for how in ("keyDown", "keyUp") for which in (held, key)]
    press = lambda key: call("POST", session + "/actions", {"actions": [{"type": "key", "id": "k", "actions": keys(key)}]})
    shot = lambda: base64.b64decode(call("GET", session + "/screenshot"))
save = lambda name: open(f"shots/{sys.argv[1]}-{name}.png", "wb").write(shot())

go(SITE + "/CNAME-control")
print("control", run("""document.documentElement.innerHTML = '<h1>control</h1><canvas></canvas>';
    const c = document.querySelector('canvas').getContext('2d'); c.fillRect(0, 0, 50, 50);
    try { return c.getImageData(0, 0, 9, 9).data[3]; } catch (e) { return `${e}`; }"""))
time.sleep(1)
save("control")
go(SITE + "/posts/specimen/#mathematics")
time.sleep(3)
save("unframed")

for name, path, width, theme, then in [
    ("home", "/", 1440, "light", ""),
    ("home-night", "/", 1440, "dark", ""),
    ("post", "/posts/c-arrays-are-not-pointers/", 1440, "light", ""),
    ("code", "/posts/specimen/#code", 1440, "light", ""),
    ("code-night", "/posts/specimen/#flake-nix", 1440, "dark", ""),
    ("wide", "/posts/specimen/#code3", 1440, "light", ""),
    ("quotes", "/posts/specimen/#quotations", 1440, "light", ""),
    ("explainer", "/posts/specimen/#mathematics", 1440, "dark", "d.querySelector('.explainer button').click()"),
    ("diagram", "/posts/specimen/", 1440, "light", "d.querySelector('.diagram').scrollIntoView()"),
    ("diagram-night", "/posts/specimen/#lookup", 1440, "dark", ""),
    ("end", "/posts/specimen/#the-rest", 1440, "light", "d.defaultView.scrollTo(0, 1e6)"),
    ("math", "/posts/notes-on-euclidean-spaces/#gram-matrix", 1440, "light", ""),
    ("tags", "/tags/", 1024, "dark", ""),
    ("about", "/about/", 1440, "light", ""),
    ("odd", "/posts/zz-odd/", 1440, "light", "d.querySelector('h2').scrollIntoView()"),
    ("math-night", "/posts/notes-on-euclidean-spaces/#norm-and-the-triangle-inequality", 1440, "dark", ""),
    ("phone-gram", "/posts/notes-on-euclidean-spaces/#gram-matrix", 320, "light", ""),
    ("phone-norm", "/posts/notes-on-euclidean-spaces/#norm-and-the-triangle-inequality", 340, "light", ""),
    ("phone-home", "/", 390, "light", ""),
    ("phone-math", "/posts/notes-on-euclidean-spaces/#proof-of-the-triangle-inequality", 390, "light", ""),
    ("phone-logic", "/posts/on-leibniz-truth/", 360, "dark", "d.querySelectorAll('.display')[4].scrollIntoView()"),
    ("phone-code", "/posts/specimen/#code", 390, "dark", "d.querySelector('.lines').scrollTo(70, 0); d.querySelector('#code1 button').click(); d.querySelector('#code1 .ref a').click()"),
]:
    go(SITE + "/colophon/")
    run(f"""localStorage.theme = '{theme}';
        const scale = Math.min(1, innerWidth / {width});
        document.documentElement.innerHTML = `<body style="margin:0;overflow:hidden"><iframe src="{path}" style="border:0;
            width:{width}px;height:${{innerHeight / scale}}px;transform:scale(${{scale}});transform-origin:0 0"></iframe>`""")
    time.sleep(3)
    print(name, json.dumps(run(CHECKS), ensure_ascii=False), flush=True)
    run("const d = document.querySelector('iframe').contentDocument;" + then)
    time.sleep(1)
    save(name)

for post in "notes-on-euclidean-spaces", "on-leibniz-truth", "specimen", "zz-math", "zz-odd":
    go(SITE + "/colophon/")
    run(f"document.documentElement.innerHTML = `<iframe src='/posts/{post}/' style='border:0;height:800px'></iframe>`")
    time.sleep(3)
    print("sweep", post, json.dumps(run(open(".github/verify/sweep.js").read()), ensure_ascii=False), flush=True)

for name, ends in [
    ("paragraph", "p, 0, p, p.childNodes.length"),
    ("from the formula", "b.lastChild, 0, f.nextSibling, 9"),
    ("back to the formula", "f.nextSibling, 9, b.lastChild, 0"),
    ("up to the formula", "p.firstChild, 0, b.firstChild, 1"),
    ("the formula", "b.firstChild, 0, b.lastChild, 1"),
    ("a displayed one", "p.querySelector('.display b').lastChild, 0, p.querySelector('.display b:last-child').firstChild, 1"),
    ("everything", "p, 0, p, 0"),
]:
    go(SITE + "/colophon/")
    go(SITE + "/posts/specimen/#mathematics")
    time.sleep(1)
    run(f"const p = document.querySelector('#mathematics + p'), f = p.querySelector('.formula'), b = f.querySelector('b'); getSelection().setBaseAndExtent({ends})")
    if name == "everything":
        press("a")
    time.sleep(1)
    state = "return [document.querySelectorAll('.picked').length, getSelection().rangeCount, scrollY].join()"
    before = run(state)
    press("c")
    time.sleep(1)
    name += f" ({before} then {run(state)})"
    run("document.body.prepend(document.createElement('textarea')); document.body.firstChild.focus()")
    press("v")
    time.sleep(1)
    print("copied", name, json.dumps(run("const it = document.body.firstChild.value; return [it.length, it.split('$').length, it.slice(-240)]")))

for theme in "light", "dark":
    print("themes", theme, run(f"""const root = document.documentElement; root.dataset.theme = '{theme}';
        return ['', 'light', 'dark', 'light !important', 'dark !important', ''].map(forced => {{
            root.style.cssText = 'color-scheme:' + forced; const s = getComputedStyle(document.body);
            return forced + ' ' + s.color + ' on ' + s.backgroundImage.match(/day|night/); }}).join(', ')"""))
print("themes toggled", run("""const root = document.documentElement; root.dataset.theme = '';
    return [1, 2, 3].map(() => (document.querySelector('.dusk').click(), root.dataset.theme || '-')).join()"""), flush=True)

go(SITE + "/colophon/")
print("feed", json.dumps(run(open(".github/verify/feed.js").read()), ensure_ascii=False), flush=True)

for theme in "light", "dark":
    go(SITE + "/colophon/")
    run(f"localStorage.theme = '{theme}'; document.documentElement.innerHTML = `<body style='margin:0'><iframe src='/posts/zz-more/' style='border:0;width:390px;height:900px'></iframe>`")
    time.sleep(3)
    start = "const w = document.querySelector('iframe').contentWindow, d = w.document, marks = d.querySelectorAll('p > .ref a'), peek = d.querySelector('.peek');"
    state = start + "return [peek.matches(':popover-open'), peek.textContent.slice(0, 12), [...d.querySelectorAll('.notes li')].map(it => it.textContent.slice(0, 5)).join(), d.activeElement.className || d.activeElement.hash, d.documentElement.scrollHeight].join(' | ')"
    print("paint", theme, run(start + """const paint = (it, ...names) => names.map(name => w.getComputedStyle(it)[name]).join();
        const inside = d.querySelector('.drawing:has(> img)');
        return [paint(d.querySelector('[fill^=color-mix]'), 'fill', 'stroke'), paint(d.querySelector('.formula [fill^=color-mix]'), 'fill'),
            paint(inside.shadowRoot.querySelector('path'), 'stroke'), inside.shadowRoot.querySelector('svg').getBoundingClientRect().width,
            paint(d.querySelector('.diagram text'), 'fill'), paint(d.querySelector('figure > .photo img, figure > img'), 'backgroundImage')].join(' | ')"""))
    save("more-" + theme)
    for name, act in [("shut", ""), ("first", "marks[0].focus(); marks[0].click()"), ("second", "marks[1].focus(); marks[1].click()"),
                      ("inner", "peek.querySelector('.ref a').click()"), ("hidden", "peek.hidePopover()"), ("again", "marks[0].click(); marks[0].click()"),
                      ("scrolled", "w.scrollBy(0, 30)")]:
        run(start + act)
        time.sleep(1)
        print("peek", name, run(state), flush=True)

go(SITE + "/posts/notes-on-euclidean-spaces/")
run("""document.querySelectorAll('link, .figure').forEach(e => e.remove());
    document.body.style.cssText = 'font: 20px serif; max-width: 40em'; document.querySelector('#gram-matrix-1, #gram-matrix').scrollIntoView()""")
time.sleep(1)
save("plain")
os._exit(0)
