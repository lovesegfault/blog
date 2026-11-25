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
const ink = c.getContext('2d').getImageData(0, 0, c.width, c.height).data.filter((v, i) => i % 4 == 3 && v).length;
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
  cut: all('main :is(p, li, h1, figure, .katex-display)').filter(e => r(e).width && (r(e).right > w.innerWidth + 1 || r(e).left < -1)).length,
  notes: all('.notes > ol > li').map(n => Math.round(r(n).top - r(d.getElementById(n.id.replace('fn', 'fnref'))).top)).join(),
  contents: all('.contents a').filter(a => a.checkVisibility()).length,
  scrolling: all('.lines').filter(l => l.scrollWidth > l.clientWidth).length,
  gutter: code && w.getComputedStyle(code).getPropertyValue('--gutter'),
  picked: picked && [picked.trim().split('\\n').length, picked.slice(0, 40)].join(),
  flat: all('.katex svg').filter(s => r(s).height < 5).length,
  orphans: all('p > .katex, li > .katex').filter(k => /^\\S/.test(k.nextSibling?.data ?? '') && line(k.nextSibling, 0).top > r(k).bottom - 4).length,
  broken: all('p > .katex, li > .katex').filter(k => k.getClientRects().length > 1).length,
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


driver = subprocess.Popen([*sys.argv[1].split(), "--port=4444"], stdout=subprocess.DEVNULL)
time.sleep(3)
started = call("POST", "/session", {"capabilities": json.loads(os.environ.get("CAPABILITIES", "{}"))})
session = "/session/" + started["sessionId"]
print(started["capabilities"]["browserName"], started["capabilities"]["browserVersion"])
print(call("POST", session + "/window/rect", {"x": 0, "y": 0, "width": 1500, "height": 1100}))
run = lambda script: call("POST", session + "/execute/sync", {"script": script, "args": []})
os.makedirs("shots", exist_ok=True)

for name, path, width, theme, then in [
    ("home", "/", 1440, "light", ""),
    ("home-night", "/", 1440, "dark", ""),
    ("post", "/posts/c-arrays-are-not-pointers/", 1440, "light", ""),
    ("code", "/posts/specimen/#code", 1440, "light", ""),
    ("code-night", "/posts/specimen/#flake-nix", 1440, "dark", ""),
    ("wide", "/posts/specimen/#code3", 1440, "light", ""),
    ("quotes", "/posts/specimen/#quotations", 1440, "light", ""),
    ("explainer", "/posts/specimen/#mathematics", 1440, "dark", "d.querySelector('.explainer button').click()"),
    ("diagram", "/posts/specimen/", 1440, "light", "d.querySelector('.diagram').scrollIntoView({block: 'center'})"),
    ("end", "/posts/specimen/#the-rest", 1440, "light", "d.defaultView.scrollTo(0, 1e6)"),
    ("math", "/posts/notes-on-euclidean-spaces/#gram-matrix", 1440, "light", ""),
    ("tags", "/tags/", 1024, "dark", ""),
    ("phone-home", "/", 390, "light", ""),
    ("phone-math", "/posts/notes-on-euclidean-spaces/", 390, "light", ""),
    ("phone-code", "/posts/specimen/#code", 390, "dark", "d.querySelector('.lines').scrollTo(70, 0); d.querySelector('#code1 button').click(); d.querySelector('#code1 .ref a').click()"),
]:
    call("POST", session + "/url", {"url": SITE + "/colophon/"})
    run(f"""localStorage.theme = '{theme}';
        const scale = Math.min(1, innerWidth / {width});
        document.documentElement.innerHTML = `<body style="margin:0;overflow:hidden"><iframe src="{path}" style="border:0;
            width:{width}px;height:${{innerHeight / scale}}px;transform:scale(${{scale}});transform-origin:0 0"></iframe>`""")
    time.sleep(3)
    print(name, json.dumps(run(CHECKS), ensure_ascii=False))
    run("const d = document.querySelector('iframe').contentDocument;" + then)
    time.sleep(1)
    open(f"shots/{name}.png", "wb").write(base64.b64decode(call("GET", session + "/screenshot")))
call("DELETE", session)
driver.terminate()
