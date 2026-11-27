"""Temporary: first visits to the About page, in fresh sessions of Safari."""

import glob
import json
import os
import subprocess
import sys
import time
import urllib.request

SITE = "http://127.0.0.1:1313"
SPY = """
window.log = [];
const mark = (what) => log.push(`${what}@${performance.now() | 0}`);
onerror = (it) => mark(it), onunhandledrejection = (it) => mark(it.reason);
const decode = HTMLImageElement.prototype.decode, context = HTMLCanvasElement.prototype.getContext;
HTMLImageElement.prototype.decode = function () {
  const name = this.src.split('/').pop().split('.')[0], done = decode.call(this);
  mark(`?${name}`), done.then(() => mark(`+${name}`), (it) => mark(`!${name} ${it} ${this.naturalWidth}`));
  return done;
};
HTMLCanvasElement.prototype.getContext = function (kind, ...rest) {
  if (kind !== 'webgl2') return context.call(this, kind, ...rest);
  for (const name of ['lost', 'restored', 'creationerror']) this.addEventListener('webglcontext' + name, (it) => mark(name + it.statusMessage));
  mark('?gl');
  const made = context.call(this, kind, ...rest);
  mark(`gl ${!!made} ${made?.isContextLost()}`);
  return made;
};
new MutationObserver(() => mark('live')).observe(document.querySelector('.bed'), { attributeFilter: ['class'] });
"""
STATE = "return [document.querySelector('.bed')?.className ?? 'REMOVED', document.visibilityState, performance.now() | 0, ...log].join(' ');"


def call(method, path, body=None):
    data = json.dumps(body).encode() if body is not None else None
    request = urllib.request.Request("http://127.0.0.1:4444" + path, data, {"Content-Type": "application/json"}, method=method)
    try:
        return json.load(urllib.request.urlopen(request))["value"]
    except urllib.error.HTTPError as error:
        return error.read().decode()[:400]


[script], [sheet] = glob.glob("result/assets/site.*.js"), glob.glob("result/assets/site.*.css")
source = open(script).read().replace("https://utteranc.es", SITE).replace("motion: reduce", "motion: never")
open(script, "w").write("{" + SPY + "}\n" + source.replace("Math.min(2, devicePixelRatio), laid", "2, laid"))
rules = open(sheet).read().replace("(prefers-reduced-motion: no-preference)", "screen")
open(sheet, "w").write(rules)
os.makedirs("shots", exist_ok=True)
driver = subprocess.Popen(["safaridriver", "--port=4444"], stdout=subprocess.DEVNULL)
server = subprocess.Popen([sys.executable, "-m", "http.server", "1313", "-b", "127.0.0.1", "-d", "result"], stderr=subprocess.DEVNULL)
for _ in range(60):
    try:
        urllib.request.urlopen(SITE, timeout=3)
        break
    except OSError:
        time.sleep(1)
try:
    for n in range(24):
        before = [[], ["/tags/"], ["/posts/home/"], ["/posts/home/", "/posts/notes-on-euclidean-spaces/"]][n % 4]
        early = [None, 0.5, 0.9][n % 3]
        session = "/session/" + call("POST", "/session", {"capabilities": {"alwaysMatch": {}}})["sessionId"]
        run = lambda script: call("POST", session + "/execute/sync", {"script": script, "args": []})
        call("POST", session + "/window/rect", {"x": 0, "y": 0, "width": 1400, "height": 900})
        for path in before:
            call("POST", session + "/url", {"url": SITE + path}), time.sleep(3.5)
            print("SEEN", n, path, run(STATE))
        call("POST", session + "/url", {"url": SITE + "/about/"})
        if early:
            time.sleep(early), call("GET", session + "/screenshot")
        time.sleep(5)
        print("SEEN", n, "ABOUT", len(before), early, run(STATE), flush=True)
        if n < 2:
            subprocess.run(["screencapture", "-x", f"shots/screen-{sys.argv[1]}-{n}.png"])
        call("DELETE", session)
finally:
    driver.kill(), server.kill()
