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
  views: all('.display').filter(f => [...f.querySelectorAll('[jax]')].filter(v => v.checkVisibility()).length != 1).length,
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
    page = browser.new_page(viewport={"width": 1440, "height": 1000})
    go, run, shot = page.goto, lambda script: page.evaluate("() => {" + script + "}"), page.screenshot
else:
    driver = subprocess.Popen([*sys.argv[1].split(), "--port=4444"], stdout=subprocess.DEVNULL)
    time.sleep(3)
    started = call("POST", "/session", {"capabilities": {}})
    session = "/session/" + started["sessionId"]
    print(started["capabilities"]["browserName"], started["capabilities"]["browserVersion"])
    print(call("POST", session + "/window/rect", {"x": 0, "y": 0, "width": 1500, "height": 1100}))
    go = lambda url: call("POST", session + "/url", {"url": url})
    run = lambda script: call("POST", session + "/execute/sync", {"script": script, "args": []})
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
os._exit(0)
