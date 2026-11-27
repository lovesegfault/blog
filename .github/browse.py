"""Temporary: looks at the lake through a WebDriver."""

import base64
import glob
import json
import os
import subprocess
import sys
import time
import urllib.request

SITE = "http://127.0.0.1:1313"
STATE = """
const bed = document.querySelector('.bed'), gl = bed?.matches('.live') && bed.getContext('webgl2');
const program = gl && gl.getParameter(gl.CURRENT_PROGRAM);
return [bed ? `${bed.className} ${bed.width}x${bed.height}` : 'removed', `dpr ${devicePixelRatio}`, `window ${innerWidth}x${innerHeight}`,
  gl && `error ${gl.getError()} linked ${!!program} drop ${program && [...gl.getUniform(program, gl.getUniformLocation(program, 'drop'))].map(it => it.toFixed(3))}`,
  getComputedStyle(document.querySelector('.figure')).getPropertyValue('--bed'), window.problems].join(' | ');
"""
CARD = """
const made = (caveat) => document.createElement('canvas').getContext('webgl2', { failIfMajorPerformanceCaveat: caveat });
const gl = made(false), info = gl?.getExtension('WEBGL_debug_renderer_info');
return `fast ${!!made(true)}, any ${!!gl}, ${gl?.getParameter(info ? info.UNMASKED_RENDERER_WEBGL : gl.RENDERER)}`;
"""
COUNT = "window.gaps = []; let then = performance.now(); (function next(now) { gaps.push(now - then); then = now; requestAnimationFrame(next); })(then);"
GAPS = "const all = gaps.splice(0).slice(2).sort((a, b) => a - b); return `frames ${all.length}, median ${all[all.length >> 1]?.toFixed(1)}, worst ${all.at(-1)?.toFixed(1)}`;"


def call(method, path, body=None):
    data = json.dumps(body).encode() if body is not None else None
    request = urllib.request.Request("http://127.0.0.1:4444" + path, data, {"Content-Type": "application/json"}, method=method)
    try:
        return json.load(urllib.request.urlopen(request))["value"]
    except urllib.error.HTTPError as error:
        return error.read().decode()[:400]


[script] = glob.glob("result/assets/site.*.js")
source = open(script, encoding="utf-8").read().replace("Caveat: true", "Caveat: false").replace("https://utteranc.es", SITE)
if "problems" not in source:
    source = "window.problems = []; onerror = (it) => problems.push(it); onunhandledrejection = (it) => problems.push(it.reason);\n" + source
source = source.replace("motion: reduce", "motion: never").replace("Math.min(2, devicePixelRatio), laid", "2, laid")
open(script, "w", encoding="utf-8").write(source)
[sheet] = glob.glob("result/assets/site.*.css")
rules = open(sheet, encoding="utf-8").read().replace("(prefers-reduced-motion: no-preference)", "screen")
open(sheet, "w", encoding="utf-8").write(rules)
os.makedirs("shots", exist_ok=True)
driver = subprocess.Popen([sys.argv[1], "--port=4444"], stdout=subprocess.DEVNULL)
server = subprocess.Popen([sys.executable, "-m", "http.server", "1313", "-b", "127.0.0.1", "-d", "result"], stderr=subprocess.DEVNULL)
try:
    time.sleep(4)
    for _ in range(30):
        try:
            urllib.request.urlopen(SITE, timeout=3)
            break
        except OSError:
            time.sleep(1)
    wanted = {
        "moz:firefoxOptions": {"prefs": {"webgl.force-enabled": True}},
        "goog:chromeOptions": {"args": ["--ignore-gpu-blocklist", "--enable-unsafe-swiftshader", "--no-sandbox"]},
    }
    started = call("POST", "/session", {"capabilities": {"alwaysMatch": {} if "safari" in sys.argv[1] else wanted}})
    session, name = "/session/" + started["sessionId"], started["capabilities"]["browserName"]
    print("SEEN", name, started["capabilities"]["browserVersion"])
    size = lambda width, height: call("POST", session + "/window/rect", {"x": 0, "y": 0, "width": width, "height": height})
    go = lambda url: call("POST", session + "/url", {"url": SITE + url})
    run = lambda script: call("POST", session + "/execute/sync", {"script": script, "args": []})
    move = lambda x, y: call("POST", session + "/actions", {"actions": [{"type": "pointer", "id": "m", "actions": [{"type": "pointerMove", "x": int(x), "y": int(y)}]}]})
    point = lambda slug: move(*run(f"const r = document.querySelector('[data-lights={slug}]').getBoundingClientRect(); return [r.x + 90, r.y + 12];"))

    def see(what, wait):
        time.sleep(wait)
        open(f"shots/{name}-{what}.png", "wb").write(base64.b64decode(call("GET", session + "/screenshot")))
        print("SEEN", name, what, run(STATE))

    size(1400, 900)
    go("/colophon/")
    print("SEEN", name, run(CARD))
    go("/posts/home/"), see("home", 9)
    run("document.querySelector('.dusk').click()"), see("night", 9), run("document.querySelector('.dusk').click()")
    go("/posts/notes-on-euclidean-spaces/"), see("notes", 9)
    go("/about/"), see("about0", 0.5), see("about1", 20), see("about2", 30)
    go("/"), time.sleep(3), run(COUNT), time.sleep(2)
    print("SEEN", name, "before", run(GAPS))
    move(700, 20), move(710, 30), time.sleep(2)
    print("SEEN", name, "loading", run(GAPS))
    point("home"), time.sleep(1.5)
    print("SEEN", name, "moving", run(GAPS))
    see("first-low", 6), point("rust-2020"), see("first-moving", 0.3), see("first-high", 8)
    size(520, 900), go("/posts/on-leibniz-truth/"), see("narrow", 9)
    call("DELETE", session)
finally:
    driver.kill(), server.kill()
